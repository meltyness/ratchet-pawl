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
    form::Form, fs::{relative, FileServer}, http::{Cookie, CookieJar, SameSite, Status}, request::{self, FromRequest}, response::status, time::Duration, tokio::sync::Mutex, Build, Request, Rocket
};

use rocket::serde::{json::Json, Serialize};

use lazy_static::lazy_static;
use serde::Deserialize;
use uuid::Uuid;
use core::str;
use std::{collections::HashMap, env, fs::File, marker::PhantomData, sync::Arc, time::{Instant, SystemTime}};

use redb::{Database, ReadableTable, TableDefinition};

const THE_DATABASE: &str = "ratchet_db.redb";
// XXX: this has to be less than i64::MAX.
const AUTH_TIMEOUT_MINUTES: u64 = 30;
/// Wraps the tables to provide type protection based on the original declaration.
struct ReadWriteTable<'a, K, V, T>(TableDefinition<'a, K, V>, PhantomData<T>) where
K: redb::Key + 'static,
V: redb::Value + 'static,
T: Serialize + Deserialize<'a> + RatchetKeyed;

impl<'a, T> ReadWriteTable<'a, &'a str, Vec<u8>, T> where T: Serialize + Deserialize<'a> + RatchetKeyed {
    pub async fn write(&'static self, item: &T) -> Result<(), redb::Error> {
        let db = Database::create(THE_DATABASE)?;
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

    pub async fn rm(&'static self, item: &T) -> Result<(), redb::Error> {
        let db = Database::create(THE_DATABASE)?;
        let write_txn = db.begin_write()?;
        {
            let mut table = write_txn.open_table(self.unwrap())?;
            let my_key = item.into_key();
            
            table.remove(my_key)?;
        }
        write_txn.commit()?;
        Ok(())
    } 

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
    static ref RATCHET_COOKIES: Mutex<HashMap<String, Instant>> = {
        let m = HashMap::new();
        Mutex::new(m)
    };
    static ref PERM_DB_KEY: Arc<&'static [u8; 32]> = {
        // SAFETY: DB_KEY must not be mutated after init, see rtp_take_key
        // if anyone else touches DB_KEY and not PERM_DB_KEY, slap them.
        unsafe { Arc::new(&DB_KEY) }
    };
}
/// SAFETY: This must only be called once prior to operation of any 
/// accessors or users of the Arc<PERM_DB_KEY>.
/// 
/// 😭 i dont want to make good abstractions right now!!
/// 
pub unsafe fn rtp_take_key (key: &String) {
    // SAFETY: Nothing else writes the DB_KEY.
    unsafe {
        DB_KEY.iter_mut()
            .enumerate()
            .for_each(|(i,k)| *k = *key.as_bytes()
                                                                .get(i)
                                                                .unwrap_or(&0));
    }
}

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

#[post("/rmuser", format = "multipart/form-data", data = "<username>")]
async fn rm_user(_admin: RatchetUser, username: Form<String>) -> status::Custom<&'static str> {
    let mut users = RATCHET_USERS.lock().await;
    match users.remove(&*username) {
        Some(user) => {
            RATCHET_USERS_TABLE.rm(&user).await.expect("Database error");
            status::Custom(Status::Ok, "")
        },
        None => status::Custom(Status::Gone, ""),
    }
}

#[post("/adduser", format = "multipart/form-data", data = "<newuser>")]
async fn add_user(_admin: RatchetUser, newuser: Form<RatchetUserEntry>) -> status::Custom<&'static str> {
    let mut users = RATCHET_USERS.lock().await;
    if !users.contains_key(&newuser.username) {
        let new_entry = RatchetUserEntry {
            username: newuser.username.clone(),
            passhash: newuser.passhash.clone(),
        };

        RATCHET_USERS_TABLE.write(&new_entry).await.expect("Database error");

        users.insert(
            newuser.username.clone(), new_entry
        );
        
        status::Custom(Status::Ok, "")
    } else {
        status::Custom(Status::Conflict, "")
    }
}

#[post("/edituser", format = "multipart/form-data", data = "<edited>")]
async fn edit_user(_admin: RatchetUser, edited: Form<RatchetUserEntry>) -> status::Custom<&'static str> {
    let mut users = RATCHET_USERS.lock().await;
    if !users.contains_key(&edited.username) {
        status::Custom(Status::Gone, "")
    } else {
        let user_update = edited.to_owned();
        RATCHET_USERS_TABLE.write(&user_update).await.expect("Database error");
        users.insert(user_update.username.clone(), user_update);
        status::Custom(Status::Ok, "")
    }
}

#[derive(Clone, FromForm, Debug, Serialize)]
struct RatchetFrontendUserEntry {
    username: String,
}

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

#[post("/rmdev", format = "multipart/form-data", data = "<network_id>")]
async fn rm_dev(_admin: RatchetUser, network_id: Form<String>) -> status::Custom<&'static str> {
    let mut devs = RATCHET_DEVICES.lock().await;
    match devs.remove(&*network_id) {
        Some(dev) => {
            RATCHET_DEVS_TABLE.rm(&dev).await.expect("Database error");
            status::Custom(Status::Ok, "")
        },
        None => status::Custom(Status::Gone, ""),
    }
}

#[post("/adddev", format = "multipart/form-data", data = "<newdev>")]
async fn add_dev(_admin: RatchetUser, newdev: Form<RatchetDevEntry>) -> status::Custom<&'static str> {
    let mut devs = RATCHET_DEVICES.lock().await;
    // TODO: Replace this with networkier stuff
    if !devs.contains_key(&newdev.network_id) {
        let new_dev = newdev.to_owned();
        RATCHET_DEVS_TABLE.write(&new_dev).await.expect("Database error");
        devs.insert(new_dev.network_id.clone(), new_dev);
        status::Custom(Status::Ok, "")
    } else {
        status::Custom(Status::Conflict, "")
    }
}

#[post("/editdev", format = "multipart/form-data", data = "<edited>")]
async fn edit_dev(_admin: RatchetUser, edited: Form<RatchetDevEntry>) -> status::Custom<&'static str> {
    let mut devs = RATCHET_DEVICES.lock().await;
    if !devs.contains_key(&edited.network_id) {
        status::Custom(Status::Gone, "")
    } else {
        let dev_update = edited.to_owned();
        RATCHET_DEVS_TABLE.write(&dev_update).await.expect("Database error");
        devs.insert(dev_update.network_id.clone(), dev_update);
        status::Custom(Status::Ok, "")
    }
}

#[derive(Clone, FromForm, Debug, Serialize)]
struct RatchetFrontendDevEntry {
    network_id: String,
}

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

#[launch]
async fn rocket() -> _ {
    rtp_force_db_init().await.expect("Unable to init database");
    rtp_import_database().await.expect("Error importing database");
    
    initialize_first_user().await.expect("Error initializing first user");
    initialize_api_key().await.expect("Error initializing API key");

    rocket::build()
        .mount("/", rocket::routes![try_login])
        .mount("/", rocket::routes![api_dump_devs, api_dump_users])
        .mount("/", rocket::routes![rm_user, edit_user, add_user, get_users])
        .mount("/", rocket::routes![rm_dev, edit_dev, add_dev, get_devs])
        .mount("/", FileServer::from(relative!("pawl-js/build/")))
        .register("/", catchers![not_found, gone, unauth, conflict])
}

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
            passhash: pass,
        };
        RATCHET_USERS_TABLE.write(&init_user).await?;
        users_init.insert(init_user.username.clone(), init_user);
    }
    Ok(())
}


async fn rtp_force_db_init() -> Result<(), redb::Error> {
    let db = Database::create(THE_DATABASE)?;

    let write_txn = db.begin_write()?;
    {
        // write initializes tables, tables must be written before they are initialized
        write_txn.open_table(RATCHET_USERS_TABLE.unwrap())?;
        write_txn.open_table(RATCHET_DEVS_TABLE.unwrap())?;
        write_txn.open_table(RATCHET_APIKEY_TABLE.unwrap())?;
        // reading an empty table is a panic.
    }
    write_txn.commit()?;
    
    let mut selected_key = {|| {for (k, v) in env::vars() {
        // something like this appears to have ok support from systemd
        if k == "RATCHET_PAWL_MASKING_KEY" { return v; }
    } return "".to_string() }}();

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

    if selected_key == "" { panic!("Please use the environment variable to specify a database encryption key."); }

    unsafe { rtp_take_key(&selected_key); }

    Ok(())
}


async fn rtp_import_database() -> Result<(), redb::Error> {
    let db = Database::create(THE_DATABASE)?;
    let mut users_init = RATCHET_USERS.lock().await;
    let mut devs_init: rocket::tokio::sync::MutexGuard<'_, HashMap<String, RatchetDevEntry>> = RATCHET_DEVICES.lock().await;
    let mut api_init = RATCHET_APIKEYS.lock().await;
    let write_txn = db.begin_write()?;
    {
        // write initializes tables, tables must be written before they are initialized
        write_txn.open_table(RATCHET_USERS_TABLE.unwrap())?;
        write_txn.open_table(RATCHET_DEVS_TABLE.unwrap())?;
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
    let table = read_txn.open_table(RATCHET_APIKEY_TABLE.unwrap())?;

    let table_iter = table.iter()?;
    table_iter.for_each(|tup| {
        let v = tup.expect("dont get this interface");
        let key = v.0.value();
        let val = v.1.value();
        let val_pt = ff.decrypt(&[], &BinaryNumeralString::from_bytes_le(&val)).unwrap().to_bytes_le();
        let val_pt = str::from_utf8(&val_pt).unwrap();
        //println!("Processing {:#?}", key);
        let new_key: RatchetApiKey = serde_json::from_str(val_pt).unwrap();
        //println!("Got {:#?}", new_key);
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

    async fn from_request(req: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        let mut cookie_store = RATCHET_COOKIES.lock().await;
        if let Some(cookie) = req.cookies().get("X-Ratchet-Auth-Token") {
            let cookie_name = cookie.value();

            match cookie_store.get_key_value(cookie_name) {
                Some((_, max_age)) if Instant::now() < *max_age => request::Outcome::Success(RatchetUser),
                Some((_, _)) => {
                    cookie_store.remove(cookie_name); // toss the cookie.
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

#[post("/trylogin", format = "multipart/form-data", data = "<creds>")]
async fn try_login(cookies: &CookieJar<'_>, creds: Form<RatchetLoginCreds>) -> status::Custom<&'static str> {
    let users = RATCHET_USERS.lock().await;
    let mut cookie_store = RATCHET_COOKIES.lock().await;
    if users.get(&creds.username)
            .unwrap_or(&RatchetUserEntry{username: "".to_string(), passhash: "".to_string()}).passhash 
        == creds.password {
        let new_uuid = Uuid::new_v4();
        let cookie = Cookie::build(("X-Ratchet-Auth-Token", new_uuid.to_string()))
                            .path("/")
                            .secure(true)
                            .max_age(Duration::minutes(AUTH_TIMEOUT_MINUTES as i64))
                            .same_site(SameSite::Lax);
                        
        cookies.add(cookie);        
        cookie_store.insert(new_uuid.to_string(), Instant::now().checked_add(std::time::Duration::from_secs(AUTH_TIMEOUT_MINUTES*60)).unwrap());
        status::Custom(Status::Ok, "")
    } else {
        status::Custom(Status::Unauthorized, "")
    }
}

#[derive(Clone, FromForm, Debug, Serialize, Deserialize)]
struct RatchetApiKey {
    api_key: String,
}

impl RatchetKeyed for RatchetApiKey{
    fn into_key(&self) -> &str {
        "-" // don't write the key in the clear
    }
}

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

    async fn from_request(req: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        let mut api_key_store = RATCHET_APIKEYS.lock().await;
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
