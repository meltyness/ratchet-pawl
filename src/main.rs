// RATCHET-pawl
//
// A rust frontend for the ratchet server 
//
// (C) 2024 - T.J. Hampton
//
#[macro_use]
extern crate rocket;

use aes::Aes256;
use fpe::ff1::{BinaryNumeralString, FF1};
use rocket::{
    form::Form, fs::{relative, FileServer}, http::{Cookie, CookieJar, SameSite, Status}, request::{self, FromRequest}, response::status, time::Duration, tokio::sync::Mutex, tokio::sync::oneshot, tokio::sync::oneshot::Sender, Request};

use rocket::serde::{json::Json, Serialize};
use rocket::{Rocket, Build};

use lazy_static::lazy_static;
use serde::Deserialize;
use uuid::Uuid;
use core::str;
use std::{collections::{HashSet, HashMap}, env, marker::PhantomData, sync::{Arc, atomic::{AtomicU64,Ordering}}, time::Instant};

use pwhash::bcrypt;

use redb::{Database, ReadableTable, TableDefinition};

use libc::{mlockall, MCL_CURRENT, MCL_FUTURE, MCL_ONFAULT};

const THE_DATABASE: &str = "ratchet_db.redb";

lazy_static! {
    static ref DB: Database = {
        Database::create(THE_DATABASE).expect("Unable to create database")
    };
}

// XXX: this has to be less than i64::MAX.
const AUTH_TIMEOUT_MINUTES: u64 = 30;

// Protect against timing / enumeration
static GUTTER: std::sync::LazyLock<Arc<rocket::tokio::sync::RwLock<String>>> = std::sync::LazyLock::new(|| Arc::new(rocket::tokio::sync::RwLock::new(String::new())));

/// Wraps the tables to provide type protection based on the original declaration.
/// 
/// Each table is associated to the same struct type (i.e., the value is always the same)
/// This enables serde to do its thing, reliably.
/// 
/// TODO:
/// - Want to squash occasionally
/// - Batching transactions would be more performant, but would likely decouple
///   the UI from success/failure feedback. Possibly something aggressive like
///   a channel that solves both neatly, tunable for disk i/o.
/// - Suspect running Database::create repeatedly probably is also a perf impact
/// 
struct ReadWriteTable<'a, K, V, T>(TableDefinition<'a, K, V>, PhantomData<T>) where
K: redb::Key + 'static,
V: redb::Value + 'static,
T: Serialize + Deserialize<'a> + RatchetKeyed;

impl<'a, T> ReadWriteTable<'a, &'a str, Vec<u8>, T> where T: Serialize + Deserialize<'a> + RatchetKeyed {
    /// Single write transaction, which due to nature of KVS includes
    /// both 'add' entry and 'modify' by way of wholesale replacement.
    pub async fn write(&'static self, item: &T) -> Result<(), redb::Error> {
        let db = &DB;
        let key: &[u8; 32]  = *PERM_DB_KEY.clone();
        let ff = FF1::<Aes256>::new(key, 2).unwrap();
        let write_txn = db.begin_write()?;
        {
            let mut table = write_txn.open_table(self.unwrap())?;
            let ser = serde_json::to_string(&item).unwrap();
            let ct = ff.encrypt(&[], &BinaryNumeralString::from_bytes_le(&ser.as_bytes())).unwrap();
            let my_key = item.into_key();
            let bytes = ct.to_bytes_le();
            table.insert(my_key, bytes.clone())?;
        }
        write_txn.commit()?;
        Ok(())
    }

    /// Single remove transaction.
    pub async fn rm(&'static self, item: &T) -> Result<(), redb::Error> {
        let db = &DB;
        let write_txn = db.begin_write()?;
        {
            let mut table = write_txn.open_table(self.unwrap())?;
            let my_key = item.into_key();
            
            table.remove(my_key)?;
        }
        write_txn.commit()?;
        Ok(())
    } 

    /// TODO: Ideally we don't have to repetedly clone this, not sure
    /// what that's doing to the vtables.
    pub fn unwrap(&self) -> TableDefinition<'static, &str, Vec<u8>> {
        self.0.to_owned()
    }
}

trait RatchetKeyed {
    fn into_key<'k>(&self) -> &str;
}

static mut DB_KEY: [u8; 32] = [0; 32];

// TODO: What happens if we modify the user format by adding a field do we have to make it option type?
const RATCHET_USERS_TABLE: ReadWriteTable<&str, Vec<u8>, RatchetUserEntry> = 
    ReadWriteTable::<&str, Vec<u8>, RatchetUserEntry>(TableDefinition::new("ratchet_users"), PhantomData);
const RATCHET_DEVS_TABLE: ReadWriteTable<&str, Vec<u8>, RatchetDevEntry> = 
    ReadWriteTable::<&str, Vec<u8>, RatchetDevEntry>(TableDefinition::new("ratchet_devs"), PhantomData);
const RATCHET_USER_CMD_POLICY_TABLE: ReadWriteTable<&str, Vec<u8>, RatchetUserCmdPolicy> = 
    ReadWriteTable::<&str, Vec<u8>, RatchetUserCmdPolicy>(TableDefinition::new("ratchet_user_cmd_policy"), PhantomData);
const RATCHET_APIKEY_TABLE: ReadWriteTable<&str, Vec<u8>, RatchetApiKey> =
    ReadWriteTable::<&str, Vec<u8>, RatchetApiKey>(TableDefinition::new("ratchet_api_keys"), PhantomData);

lazy_static! {
    static ref RATCHET_APIKEYS: Mutex<HashMap<String, RatchetApiKey>> = {
        let m = HashMap::new();
        Mutex::new(m)
    };
    static ref RATCHET_USERS: Mutex<HashMap<String, RatchetUserEntry>> = {
        let m = HashMap::new();
        Mutex::new(m)
    };
    static ref RATCHET_DEVICES: Mutex<HashMap<String, RatchetDevEntry>> = {
        let m = HashMap::new();
        Mutex::new(m)
    };
    static ref RATCHET_USER_CMD_POLICY: Mutex<RatchetUserCmdPolicy> = {
        let s = String::new();
        Mutex::new(RatchetUserCmdPolicy(s))
    };
    // TODO: It's pretty risky to leave these as separate mutex
    static ref RATCHET_COOKIES: Mutex<HashMap<String, (Instant, String)>> = {
        let m = HashMap::new();
        Mutex::new(m)
    };
    static ref RATCHET_USER_COOKIES: Mutex<HashMap<String, HashSet<String>>> = {
        let m = HashMap::new();
        Mutex::new(m)
    };
    static ref RATCHET_POLL_PINS: Mutex<Vec<Sender<bool>>> = {
        let p = Vec::new();
        Mutex::new(p)
    };
    static ref PERM_DB_KEY: Arc<&'static [u8; 32]> = {
        // SAFETY: DB_KEY must not be mutated after init, see rtp_take_key
        // if anyone else touches DB_KEY and not PERM_DB_KEY, slap them.
        unsafe {
            #[allow(static_mut_refs)]
            Arc::new(&DB_KEY) 
        }
    };
    // Recommend polling upon attach to subscribers.
    static ref LONG_POLL_EPOCH: AtomicU64 = AtomicU64::new(1);
}

/// SAFETY: This must only be called once prior to operation of any 
/// accessors or users of the Arc<PERM_DB_KEY>.
/// 
/// 😭 i dont want to make good abstractions right now!!
/// I think this is actually a 'std::LazyLock' maybe use it 
/// 
pub unsafe fn rtp_take_key (key: &String) {
    // SAFETY: Nothing else writes the DB_KEY.
    unsafe {
        #[allow(static_mut_refs)]
        DB_KEY.iter_mut()
            .enumerate()
            .for_each(|(i,k)| *k = *key.as_bytes()
                                                                .get(i)
                                                                .unwrap_or(&0));
    }
}

/// Backend data for users, used for authentication
/// 
/// Should not be sent over any unsecure channel, since
/// hashes are subject to attacks.
/// 
#[derive(Clone, FromForm, Debug, Serialize, Deserialize)]
struct RatchetUserEntry {
    username: String,
    passhash: String,
}

impl RatchetKeyed for RatchetUserEntry{
    fn into_key(&self) -> &str {
        &self.username.as_str()
    }
}

/// For long-pollers, each API change results in sending a notification out.
/// This notificaation is associated with a serial number for the total number of
/// changes seen so far.
/// 
/// Then on re-attach, a poller who may have missed multiple updates is 
/// immediately notified again, rather than leaving open the possibility of 
/// missed updates, and a subscriber in a stale state.
/// 
/// It's also really really cheap.
/// 
/// TODO: Frontend pollers for AJAX / real-time updates.
/// 
async fn rtp_notify_pollers() {
    LONG_POLL_EPOCH.fetch_add(1, Ordering::Relaxed);
    let mut pins = RATCHET_POLL_PINS.lock().await;
    while let Some(pin) = pins.pop() {
        let _ = pin.send(true); // it's pub-sub, so it will leak; 
                        // we don't know the subscriber is gone until after they fail to reconnect.
    }
}

/// Frontend API for removing a user by username.
/// 
/// TODO: Don't remove the bottom dollar
/// 
#[post("/rmuser", format = "multipart/form-data", data = "<username>")]
async fn rm_user(_admin: RatchetUser, username: Form<String>) -> status::Custom<&'static str> {
    let mut users = RATCHET_USERS.lock().await;
    // and deauthorize from web shell
    let mut cookie_store = RATCHET_COOKIES.lock().await;
    let mut user_cookies = RATCHET_USER_COOKIES.lock().await;
    match users.remove(&*username) {
        Some(user) => {
            RATCHET_USERS_TABLE.rm(&user).await.expect("Database error");
            match user_cookies.remove(&user.username) {
                Some(active_cookies) => {
                    active_cookies.into_iter().for_each(|each_cookie| {cookie_store.remove(&each_cookie);});
                },
                None => (),
            }
            rocket::tokio::spawn(rtp_notify_pollers());
            status::Custom(Status::Ok, "")
        },
        None => status::Custom(Status::Gone, ""),
    }
}

/// Frontend API for adding a user.
/// 
/// TODO: Input validation, password policy
/// 
#[post("/adduser", format = "multipart/form-data", data = "<newuser>")]
async fn add_user(_admin: RatchetUser, newuser: Form<RatchetUserEntry>) -> status::Custom<&'static str> {
    let mut users = RATCHET_USERS.lock().await;
    if !users.contains_key(&newuser.username) {
        if let Ok(h) = bcrypt::hash(&newuser.passhash) {
            let new_entry = RatchetUserEntry {
                username: newuser.username.clone(),
                passhash: h.clone(),
            };

            RATCHET_USERS_TABLE.write(&new_entry).await.expect("Database error");

            users.insert(
                newuser.username.clone(), new_entry
            );
            rocket::tokio::spawn(rtp_notify_pollers());
            status::Custom(Status::Ok, "")
        } else {
            // TODO: This doesn't exactly mean this anymore
            status::Custom(Status::Conflict, "")
        }
    } else {
        status::Custom(Status::Conflict, "")
    }
}

/// Frontend API for editing a user.
/// 
/// TODO: Input validation, password policy
/// 
#[post("/edituser", format = "multipart/form-data", data = "<edited>")]
async fn edit_user(_admin: RatchetUser, edited: Form<RatchetUserEntry>) -> status::Custom<&'static str> {
    let mut users = RATCHET_USERS.lock().await;
    // and deauthorize from web shell
    let mut cookie_store = RATCHET_COOKIES.lock().await;
    let mut user_cookies = RATCHET_USER_COOKIES.lock().await;
    if !users.contains_key(&edited.username) {
        status::Custom(Status::Gone, "")
    } else {
        let mut user_update = edited.to_owned();
        if let Ok(h) = bcrypt::hash(user_update.passhash)  {
            user_update.passhash = h;
            RATCHET_USERS_TABLE.write(&user_update).await.expect("Database error");
            users.insert(user_update.username.clone(), user_update.clone());

            match user_cookies.remove(&user_update.username) {
                Some(active_cookies) => {
                    active_cookies.into_iter().for_each(|each_cookie| {cookie_store.remove(&each_cookie);});
                },
                None => (),
            }
            rocket::tokio::spawn(rtp_notify_pollers());
            status::Custom(Status::Ok, "")
        } else {
            // TODO: This doesn't exactly mean this anymore
            status::Custom(Status::Gone, "")
        }
    }
}

/// Special structure to only return safe userdata
/// back to the Frontend.
#[derive(Clone, FromForm, Debug, Serialize)]
struct RatchetFrontendUserEntry {
    username: String,
}

/// Frontend API for listing users.
#[get("/getusers")]
async fn get_users(_admin: RatchetUser) -> Json<Vec<RatchetFrontendUserEntry>> {
    let users = RATCHET_USERS.lock().await;
    Json(
        users
            .values()
            .map(|u| RatchetFrontendUserEntry {
                username: u.username.clone(),
            })
            .collect::<Vec<RatchetFrontendUserEntry>>(),
    )
}

/// Backend data for TACACS+ clients, used for authentication
/// 
/// Should not be sent over any unsecure channel, since
/// hashes are subject to attacks.
/// 
#[derive(Clone, FromForm, Debug, Serialize, Deserialize)]
struct RatchetDevEntry {
    network_id: String,
    key: String,
}

impl RatchetKeyed for RatchetDevEntry{
    fn into_key(&self) -> &str {
        self.network_id.as_str()
    }
}

/// Frontend API for removing devices.
#[post("/rmdev", format = "multipart/form-data", data = "<network_id>")]
async fn rm_dev(_admin: RatchetUser, network_id: Form<String>) -> status::Custom<&'static str> {
    let mut devs = RATCHET_DEVICES.lock().await;
    match devs.remove(&*network_id) {
        Some(dev) => {
            RATCHET_DEVS_TABLE.rm(&dev).await.expect("Database error");
            rocket::tokio::spawn(rtp_notify_pollers());
            status::Custom(Status::Ok, "")
        },
        None => status::Custom(Status::Gone, ""),
    }
}

/// Frontend API for adding devices.
///  
/// TODO: Input validation, password policy
/// 
#[post("/adddev", format = "multipart/form-data", data = "<newdev>")]
async fn add_dev(_admin: RatchetUser, newdev: Form<RatchetDevEntry>) -> status::Custom<&'static str> {
    let mut devs = RATCHET_DEVICES.lock().await;
    // TODO: Replace this with networkier stuff
    if !devs.contains_key(&newdev.network_id) {
        let new_dev = newdev.to_owned();
        RATCHET_DEVS_TABLE.write(&new_dev).await.expect("Database error");
        devs.insert(new_dev.network_id.clone(), new_dev);
        rocket::tokio::spawn(rtp_notify_pollers());
        status::Custom(Status::Ok, "")
    } else {
        status::Custom(Status::Conflict, "")
    }
}

/// Frontend API for editing devices.
///  
/// TODO: Input validation, password policy
/// 
#[post("/editdev", format = "multipart/form-data", data = "<edited>")]
async fn edit_dev(_admin: RatchetUser, edited: Form<RatchetDevEntry>) -> status::Custom<&'static str> {
    let mut devs = RATCHET_DEVICES.lock().await;
    if !devs.contains_key(&edited.network_id) {
        status::Custom(Status::Gone, "")
    } else {
        let dev_update = edited.to_owned();
        RATCHET_DEVS_TABLE.write(&dev_update).await.expect("Database error");
        devs.insert(dev_update.network_id.clone(), dev_update);
        rocket::tokio::spawn(rtp_notify_pollers());
        status::Custom(Status::Ok, "")
    }
}

/// Special structure to only return safe devdata
/// back to the Frontend.
#[derive(Clone, FromForm, Debug, Serialize)]
struct RatchetFrontendDevEntry {
    network_id: String,
}

/// Frontend API for listing users.
#[get("/getdevs")]
async fn get_devs(_admin: RatchetUser) -> Json<Vec<RatchetFrontendDevEntry>> {
    let devs = RATCHET_DEVICES.lock().await;
    Json(
        devs.values()
            .map(|d| RatchetFrontendDevEntry {
                network_id: d.network_id.clone(),
            })
            .collect::<Vec<RatchetFrontendDevEntry>>(),
    )
}

/// Frontend API for dumping policy.
#[get("/getpolicy")]
async fn get_policy(_admin: RatchetUser) -> String {
    RATCHET_USER_CMD_POLICY.lock().await.0.clone()
}

/// Backend API for getting user creds.
#[get("/api/dumpusers")]
async fn api_dump_users(_valid: RatchetApiKey) -> String {
    let users = RATCHET_USERS.lock().await;
    users.iter().fold(
        String::new(),
        |mut resp, (user, hash)| {
            resp.push_str(user);
            resp.push_str(",");
            resp.push_str(&hash.passhash);
            resp.push_str("\n");
            resp
        }
    )
}

/// Backend API for getting dev keys.
#[get("/api/dumpdevs")]
async fn api_dump_devs(_valid: RatchetApiKey) -> String {
    let devs = RATCHET_DEVICES.lock().await;
    devs.iter().fold(
        String::new(),
        |mut resp, (network, key)| {
            resp.push_str(network);
            resp.push_str(",");
            resp.push_str(&key.key);
            resp.push_str("\n");
            resp
        }
    )
}

/// Backend API for signalling updates.
#[get("/api/longpoll?<serial>")]
async fn api_long_poll(_valid: RatchetApiKey, serial: Option<u64>) -> String {
    // Serial number mismatch
    let latest = LONG_POLL_EPOCH.load(Ordering::Relaxed);
    if let Some(sn) = serial  { // Option retains compatibility with legacy ratchet
        if sn != latest { return String::from(format!("Update {}", latest)); } // output
    }

    // Normal path / waiting
    let (tx, rx) = oneshot::channel();
    {
        let mut pins = RATCHET_POLL_PINS.lock().await;
        pins.push(tx);
    } // drop the pins

    match rx.await {
        Ok(_v) => String::from(format!("Update {}", latest)),
        Err(_) => String::from(""),
    }
}

#[catch(401)]
fn unauth(_req: &Request) -> String {
    format!("Unauthorized")
}
#[catch(404)]
fn not_found(_req: &Request) -> String {
    format!("Not Found")
}
#[catch(409)]
fn gone(_req: &Request) -> String {
    format!("Gone")
}
#[catch(410)]
fn conflict(_req: &Request) -> String {
    format!("Conflict")
}
// #[catch(503)]
// fn svc_unavailable(_req: &Request) -> String {
//     format!("Service Unavailable")
// }

fn main() {
    // lock all allocations
    #[cfg(not(debug_assertions))]
    {
        let result = unsafe { mlockall(MCL_CURRENT | MCL_FUTURE | MCL_ONFAULT) };
        if result != 0 {
            eprintln!("mlockall failed with error code: {}", result);
        } else {
            println!("mlockall succeeded");
        }
    }
    // https://github.com/rwf2/Rocket/issues/1881 👍👍👍
    rocket::execute(async move {
            let _ = rocket().await
            .launch()
            .await;
        });
}

async fn rocket() -> Rocket<Build> {
    rtp_force_db_init().await.expect("Unable to init database");
    rtp_import_database().await.expect("Error importing database");
    
    initialize_first_user().await.expect("Error initializing first user");
    initialize_user_cmd_pol().await.expect("Error initializing user cmd policy");
    initialize_api_key().await.expect("Error initializing API key");

    rt_generate_gutter().await;

    rocket::build()
        .mount("/", rocket::routes![try_login, logged, hangup])
        .mount("/", rocket::routes![api_dump_devs, api_dump_users, api_long_poll])
        .mount("/", rocket::routes![rm_user, edit_user, add_user, get_users])
        .mount("/", rocket::routes![rm_dev, edit_dev, add_dev, get_devs])
        .mount("/",rocket::routes![get_policy])
        .mount("/", FileServer::from(relative!("pawl-js/build/")))
        .register("/", catchers![not_found, gone, unauth, conflict])
}

async fn rt_generate_gutter() {
    let mut g = GUTTER.write().await;
    g.push_str(&bcrypt::hash(rt_generate_gutter_string()).expect("Ratchet Fatal: Unable to generate gutter"));
}

fn rt_generate_gutter_string() -> String { 
    (0..72).fold(
        String::with_capacity(72),
        |mut s, _| {
            loop {
                let c = rand::random::<u8>();
                if c.is_ascii_alphanumeric() || c.is_ascii_graphic() || c.is_ascii_punctuation() {
                    s.push(c as char);
                    break;
                }
            }
            s
        }
    )
}

#[derive(Clone, FromForm, Debug, Serialize, Deserialize)]
struct RatchetUserCmdPolicy(String);
impl RatchetKeyed for RatchetUserCmdPolicy {
    fn into_key<'k>(&self) -> &str {
        "singleton"
    }
}

/// An invariant that is largely maintained throughout is that
/// there is at least one user who can administer ratchet in the database.
async fn initialize_user_cmd_pol() -> Result<(), redb::Error> {
    let mut user_cmd_policy_init = RATCHET_USER_CMD_POLICY.lock().await;
    if user_cmd_policy_init.0.len() == 0 {
        *user_cmd_policy_init = RatchetUserCmdPolicy(String::from("$\n(\n)"));
        RATCHET_USER_CMD_POLICY_TABLE.write(&user_cmd_policy_init).await;
    }
    Ok(())
}

/// An invariant that is largely maintained throughout is that
/// there is at least one user who can administer ratchet in the database.
async fn initialize_first_user() -> Result<(), redb::Error> {
    let mut users_init = RATCHET_USERS.lock().await;
    if users_init.len() == 0 {
        let mut pass: String = String::with_capacity(16);
        while pass.len() < 16 {
            let c = rand::random::<u8>();
            if c.is_ascii_alphanumeric() || c.is_ascii_graphic() || c.is_ascii_punctuation() {
                pass.push(c as char);
            }
        }
        println!("Ratchet-Pawl Initialization creating initial user with details:");
        println!("Username: DefaultRatchetUser");
        println!("Password: {}", pass);
        let username = String::from("DefaultRatchetUser");
        let init_user = RatchetUserEntry {
            username: username,
            passhash: bcrypt::hash(pass).expect("unable to initialize password"),
        };
        RATCHET_USERS_TABLE.write(&init_user).await?;
        users_init.insert(init_user.username.clone(), init_user);
    }
    Ok(())
}

/// redb doesn't write any tables until you open them.
/// this ensures that the needed tables exist in the
/// database.
async fn rtp_force_db_init() -> Result<(), redb::Error> {
    let db = &DB;

    let write_txn = db.begin_write()?;
    {
        // write initializes tables, tables must be written before they are initialized
        write_txn.open_table(RATCHET_USERS_TABLE.unwrap())?;
        write_txn.open_table(RATCHET_DEVS_TABLE.unwrap())?;
        write_txn.open_table(RATCHET_USER_CMD_POLICY_TABLE.unwrap())?;
        write_txn.open_table(RATCHET_APIKEY_TABLE.unwrap())?;
        // reading an empty table is a panic.
    }
    write_txn.commit()?;
    
    let selected_key = {|| {for (k, v) in env::vars() {
        // something like this appears to have ok support from systemd
        if k == "RATCHET_PAWL_MASKING_KEY" { return v; }
    } return "".to_string() }}();

    if selected_key == "" { panic!("Please use the environment var, RATCHET_PAWL_MASKING_KEY, to specify a database encryption key."); }
    // if selected_key == "" {
    //     // ZERG RUSH!!
    //     let ke = File::open(THE_DATABASE)?;
    //     let ke = ke.metadata()?;
    //     let ke = ke.created()?;
    //     let ke = ke.duration_since(SystemTime::UNIX_EPOCH);
    //     selected_key = match ke {
    //         Ok(k) => k.as_secs().to_string(),
    //         Err(_) => panic!("Need a shadowing key."),
    //     };
    // }


    unsafe { rtp_take_key(&selected_key); }

    Ok(())
}

/// This is the mechanism that puts the database in memory.
/// 
/// pawl ensures that its hash tables always exactly match
/// the contents of the database, and that all clients are
/// eventually consistent with the status.
async fn rtp_import_database() -> Result<(), redb::Error> {
    let db = &DB;
    let mut users_init = RATCHET_USERS.lock().await;
    let mut devs_init: rocket::tokio::sync::MutexGuard<'_, HashMap<String, RatchetDevEntry>> = RATCHET_DEVICES.lock().await;
    let mut user_cmd_policy_init = RATCHET_USER_CMD_POLICY.lock().await;
    let mut api_init = RATCHET_APIKEYS.lock().await;
    let write_txn = db.begin_write()?;
    {
        // write initializes tables, tables must be written before they are initialized
        write_txn.open_table(RATCHET_USERS_TABLE.unwrap())?;
        write_txn.open_table(RATCHET_DEVS_TABLE.unwrap())?;
        write_txn.open_table(RATCHET_USER_CMD_POLICY_TABLE.unwrap())?;
        write_txn.open_table(RATCHET_APIKEY_TABLE.unwrap())?;
        // reading an empty table is a panic.
    }
    write_txn.commit()?;

    let key: &[u8; 32]  = *PERM_DB_KEY.clone();
    let ff = FF1::<Aes256>::new(key, 2).unwrap();

    let read_txn = db.begin_read()?;
    let table = read_txn.open_table(RATCHET_USERS_TABLE.unwrap())?;

    let table_iter = table.iter()?;
    table_iter.for_each(|tup| {
        let v = tup.expect("dont get this interface");
        let key = v.0.value();
        let val = v.1.value(); // these are shadowed already and not directly recoverable. but we're just gonna encrypt it anyway.
        let val_pt = ff.decrypt(&[], &BinaryNumeralString::from_bytes_le(&val)).unwrap().to_bytes_le();
        let val_pt = str::from_utf8(&val_pt).unwrap();
        //println!("Processing {:#?}", key);
        let new_user: RatchetUserEntry = serde_json::from_str(val_pt).unwrap();
        //println!("Got {:#?}", new_user);
        users_init.insert(key.to_string(), new_user);
    });

    let key: &[u8; 32]  = *PERM_DB_KEY.clone();
    let ff = FF1::<Aes256>::new(key, 2).unwrap();

    let read_txn = db.begin_read()?;
    let table = read_txn.open_table(RATCHET_DEVS_TABLE.unwrap())?;
 
    let table_iter = table.iter()?;
    table_iter.for_each(|tup| {
        let v = tup.expect("dont get this interface");
        let record_key = v.0.value();
        let val = v.1.value();
        let val_pt = ff.decrypt(&[], &BinaryNumeralString::from_bytes_le(&val)).unwrap().to_bytes_le();
        let val_pt = str::from_utf8(&val_pt).unwrap();
        //println!("Processing {:#?}", key);
        let new_dev: RatchetDevEntry = serde_json::from_str(&val_pt).unwrap();
        //println!("Got {:#?}", new_dev);
        devs_init.insert(record_key.to_string(), new_dev);
    });

    let read_txn = db.begin_read()?;
    let table = read_txn.open_table(RATCHET_USER_CMD_POLICY_TABLE.unwrap())?;
    let table_iter = table.iter()?;
    table_iter.for_each(|tup| {
        let v = tup.expect("dont get this interface");
        let val = v.1.value();
        let val_pt = ff.decrypt(&[], &BinaryNumeralString::from_bytes_le(&val)).unwrap().to_bytes_le();
        let val_pt = str::from_utf8(&val_pt).unwrap();

        *user_cmd_policy_init = serde_json::from_str(&val_pt).unwrap();
    });    

    let read_txn = db.begin_read()?;
    let table = read_txn.open_table(RATCHET_APIKEY_TABLE.unwrap())?;

    let table_iter = table.iter()?;
    table_iter.for_each(|tup| {
        let v = tup.expect("dont get this interface");
        //let key = v.0.value();
        let val = v.1.value();
        let val_pt = ff.decrypt(&[], &BinaryNumeralString::from_bytes_le(&val)).unwrap().to_bytes_le();
        let val_pt = str::from_utf8(&val_pt).unwrap();
        //println!("Processing {:#?}", key);
        let new_key: RatchetApiKey = serde_json::from_str(val_pt).unwrap();
        //println!("Got {:#?}", new_key);
        // CONTRACT: ratchet-cycle intermediates pawl and ratchet to deliver this
        println!("Api-Key: {}", new_key.api_key.clone());
        api_init.insert(new_key.api_key.clone(), new_key); // this awkward bit is because write is genuinely key-value
    });

    Ok(())
}

struct RatchetUser;
enum RatchetAuthError {
    NotAuthenticated
}

impl std::fmt::Debug for RatchetAuthError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Authentication error, unknown user")
    }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for RatchetUser {
    type Error = RatchetAuthError;
    /// Mechanism to identify whether someone who posesses
    /// a cookies has an authorized cookie or not.
    async fn from_request(req: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        let mut cookie_store = RATCHET_COOKIES.lock().await;
        let mut user_cookies = RATCHET_USER_COOKIES.lock().await;
        if let Some(cookie) = req.cookies().get("X-Ratchet-Auth-Token") {
            let cookie_name = cookie.value();

            match cookie_store.get_key_value(cookie_name) {
                Some((_, max_age)) if Instant::now() < max_age.0 => request::Outcome::Success(RatchetUser),
                Some((_, _)) => {
                    if let Some((_, associated_user)) = cookie_store.remove(cookie_name){ // toss the cookie.
                        match user_cookies.get_mut(&associated_user) {
                            Some(cs) => {
                                cs.remove(cookie_name);
                                if cs.len() == 0 { user_cookies.remove(&associated_user); }
                            },
                            None => (),
                        }
                    }
                    request::Outcome::Error((Status::Unauthorized, RatchetAuthError::NotAuthenticated))
                 },
                _ => request::Outcome::Error((Status::Unauthorized, RatchetAuthError::NotAuthenticated))
            }
        } else {
            // bugger off
            request::Outcome::Error((Status::Unauthorized, RatchetAuthError::NotAuthenticated))
        }
    }
}

#[derive(Clone, FromForm)]
struct RatchetLoginCreds {
    username: String,
    password: String,
}

/// TODO: Move out west and do something with JWT
#[post("/trylogin", format = "multipart/form-data", data = "<creds>")]
async fn try_login(cookies: &CookieJar<'_>, creds: Form<RatchetLoginCreds>) -> status::Custom<&'static str> {
    let users = RATCHET_USERS.lock().await;
    let mut cookie_store = RATCHET_COOKIES.lock().await;
    let mut user_cookies = RATCHET_USER_COOKIES.lock().await;
    if bcrypt::verify(&creds.password, &users.get(&creds.username).unwrap_or(&RatchetUserEntry{ username: "".to_string(), passhash: (&GUTTER.read().await).to_string() }).passhash) && users.contains_key(&creds.username) {
        let new_uuid = Uuid::new_v4();
        let cookie = Cookie::build(("X-Ratchet-Auth-Token", new_uuid.to_string()))
                            .path("/")
                            .secure(true)
                            .max_age(Duration::minutes(AUTH_TIMEOUT_MINUTES as i64))
                            .same_site(SameSite::Strict);
                        
        cookies.add(cookie); 
        let timeout = Instant::now() + std::time::Duration::from_secs(AUTH_TIMEOUT_MINUTES*60);
        cookie_store.insert(new_uuid.to_string(), (timeout,creds.username.clone()));
        match user_cookies.get_mut(&creds.username) {
            Some(h) => {
                h.insert(new_uuid.to_string());
            },
            None => {
                let mut h = HashSet::new();
                h.insert(new_uuid.to_string());
                user_cookies.insert(creds.username.clone(), h);
            },
        }
        // Also schedule a task to delete the cookie around the same time as the timeout
        // deauthorizing it.
        rocket::tokio::spawn(wipe_cookie(creds.username.clone(), new_uuid.to_string(), timeout));
        status::Custom(Status::Ok, "")
    } else {
        status::Custom(Status::Unauthorized, "")
    }
}

/// When the timeout has lapsed, the cookie is removed from the table, and no longer authorized
/// Additionally, if the user has no more cookies
async fn wipe_cookie(name: String, uuid: String, when: Instant) {
    rocket::tokio::time::sleep_until(rocket::tokio::time::Instant::from_std(when)).await;
    {
        let mut cookie_store = RATCHET_COOKIES.lock().await;
        let mut user_cookies = RATCHET_USER_COOKIES.lock().await;
        cookie_store.remove(&uuid);
        match user_cookies.get_mut(&name) {
            Some(u) => { 
                u.remove(&uuid);
                if u.len() == 0 {
                    user_cookies.remove(&name); // TODO: Encapsulate the maintenance of multiple user cookies
                                                //  add policies for max simultaneous sessions, etc.
                }
            },
            None => (),
        }
    }
}

/// The frontend needs to know if the user is still authenticated so that
/// data loss isn't encountered, when avoidable.
#[get("/logged")]
async fn logged(_admin: RatchetUser) -> status::Custom<&'static str> {
    status::Custom(Status::Ok, "")
}

/// Users may want to log out and log back in to guarantee 30 more minutes of
/// installing users whose names are all just floating point values as fast
/// as disk / I/O contention permit.
#[get("/hangup")]
async fn hangup(_admin: RatchetUser, cookies: &CookieJar<'_>) -> status::Custom<&'static str> {
    if let Some(c) = cookies.get("X-Ratchet-Auth-Token") {
        let mut cookie_store = RATCHET_COOKIES.lock().await;
        let mut user_cookies = RATCHET_USER_COOKIES.lock().await;
        let cookie_name = c.value();
        if let Some((_, associated_user)) = cookie_store.remove(cookie_name) { // toss the cookie.
            match user_cookies.get_mut(&associated_user) {
                Some(cs) => {
                    cs.remove(cookie_name);
                    if cs.len() == 0 {
                        user_cookies.remove(&associated_user);
                    }
                },
                None => (),
            }
        }

    }
    status::Custom(Status::Ok, "")
}

/// The API Key is for the backend / ratchet-proper to fetch details about
/// the authentication / authorization database.
#[derive(Clone, FromForm, Debug, Serialize, Deserialize)]
struct RatchetApiKey {
    api_key: String,
}

impl RatchetKeyed for RatchetApiKey{
    fn into_key(&self) -> &str {
        "-" // don't write the key in the clear
    }
}

/// Choose a pretty hard-to-guess API key
async fn initialize_api_key() -> Result<(), redb::Error> { 
    let mut api_key: String = String::with_capacity(128);
    let mut api_init = RATCHET_APIKEYS.lock().await;
    if api_init.len() == 0 {
        while api_key.len() < 128 {
            let c = rand::random::<u8>();
            if c.is_ascii_alphanumeric() || c.is_ascii_graphic() || c.is_ascii_punctuation() {
                api_key.push(c as char);
            }
        }
        println!("Ratchet-Pawl Initialization creating API-Key details:");
        // CONTRACT: ratchet-cycle expects the api-key to be dumped in the first 10 or so 
        // lines for pawl's execution, make sure to maintain this; it matches on "Api-Key: " pattern
        println!("Api-Key: {}", api_key);
        api_init.insert(api_key.clone(), RatchetApiKey {
                api_key: api_key.clone()
            });
        RATCHET_APIKEY_TABLE.write(&RatchetApiKey {
            api_key: api_key
        }).await?
    }
    Ok(())
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for RatchetApiKey {
    type Error = RatchetAuthError;
    /// Mechanism to identify an API user.
    async fn from_request(req: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        let api_key_store = RATCHET_APIKEYS.lock().await;
        if let Some(api_key) = req.headers().get_one("X-Ratchet-Api-Key") {
            match api_key_store.get_key_value(api_key) {
                Some((_, _)) => {
                    request::Outcome::Success(RatchetApiKey{api_key: "".to_string()})
                 },
                _ => request::Outcome::Error((Status::NotFound, RatchetAuthError::NotAuthenticated))
            }
        } else {
            // bugger off
            request::Outcome::Error((Status::NotFound, RatchetAuthError::NotAuthenticated))
        }
    }
}
