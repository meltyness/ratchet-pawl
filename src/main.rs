// RATCHET-pawl
//
// A rust frontend for the ratchet server 
//
// (C) 2024 - T.J. Hampton
//
#[macro_use]
extern crate rocket;

use rocket::{
    form::Form,
    fs::{relative, FileServer},
    http::Status,
    response::status,
    tokio::sync::Mutex,
    Build, Request, Rocket,
};

use rocket::serde::{json::Json, Serialize};

use lazy_static::lazy_static;
use serde::Deserialize;
use std::{collections::HashMap, marker::PhantomData};

use redb::{Database, ReadableTable, TableDefinition};

const THE_DATABASE: &str = "ratchet_db.redb";

/// Wraps the tables to provide type protection based on the original declaration.
struct ReadWriteTable<'a, K, V, T>(TableDefinition<'a, K, V>, PhantomData<T>) where
K: redb::Key + 'static,
V: redb::Value + 'static,
T: Serialize + Deserialize<'a> + RatchetKeyed;

impl<'a, T> ReadWriteTable<'a, &'a str, &'a str, T> where T: Serialize + Deserialize<'a> + RatchetKeyed {
    pub async fn write(&'static self, item: &T) -> Result<(), redb::Error> {
        let db = Database::create(THE_DATABASE)?;
        let write_txn = db.begin_write()?;
        {
            let mut table = write_txn.open_table(self.unwrap())?;
            let ser = serde_json::to_string(&item).unwrap();
            let my_key = item.into_key();
            
            table.insert(my_key, ser.as_str())?;
        }
        write_txn.commit()?;
        Ok(())
    }

    pub async fn rm(&'static self, item: &T) -> Result<(), redb::Error>{
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

    pub fn unwrap(&self) -> TableDefinition<'static, &str, &str> {
        self.0.to_owned()
    }
}

trait RatchetKeyed {
    fn into_key<'k>(&self) -> &str;
}

// TODO: What happens if we modify the user format by adding a field do we have to make it option type?
const RATCHET_USERS_TABLE: ReadWriteTable<&str, &str, RatchetUserEntry> = 
    ReadWriteTable::<&str, &str, RatchetUserEntry>(TableDefinition::new("ratchet_users"), PhantomData);
const RATCHET_DEVS_TABLE: ReadWriteTable<&str, &str, RatchetDevEntry> = 
    ReadWriteTable::<&str, &str, RatchetDevEntry>(TableDefinition::new("ratchet_devs"), PhantomData);

lazy_static! {
    static ref RATCHET_USERS: Mutex<HashMap<String, RatchetUserEntry>> = {
        let m = HashMap::new();
        Mutex::new(m)
    };
    static ref RATCHET_DEVICES: Mutex<HashMap<String, RatchetDevEntry>> = {
        let m = HashMap::new();
        Mutex::new(m)
    };
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
async fn rm_user(username: Form<String>) -> status::Custom<&'static str> {
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
async fn add_user(newuser: Form<RatchetUserEntry>) -> status::Custom<&'static str> {
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
async fn edit_user(edited: Form<RatchetUserEntry>) -> status::Custom<&'static str> {
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
async fn get_users() -> Json<Vec<RatchetFrontendUserEntry>> {
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
async fn rm_dev(network_id: Form<String>) -> status::Custom<&'static str> {
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
async fn add_dev(newdev: Form<RatchetDevEntry>) -> status::Custom<&'static str> {
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
async fn edit_dev(edited: Form<RatchetDevEntry>) -> status::Custom<&'static str> {
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
async fn get_devs() -> Json<Vec<RatchetFrontendDevEntry>> {
    let devs = RATCHET_DEVICES.lock().await;
    Json(
        devs.values()
            .map(|d| RatchetFrontendDevEntry {
                network_id: d.network_id.clone(),
            })
            .collect::<Vec<RatchetFrontendDevEntry>>(),
    )
}

#[catch(404)]
fn not_found(_req: &Request) -> String {
    format!("Not Found")
}

#[launch]
async fn rocket() -> _ {
    rtp_import_database().await.expect("Error importing database");

    add_dev_routes(add_user_routes(
        rocket::build()
            .mount("/", FileServer::from(relative!("pawl-js/build/")))
            .register("/", catchers![not_found]),
    ))
}

async fn rtp_import_database() -> Result<(), redb::Error> {
    let db = Database::create(THE_DATABASE)?;
    let mut users_init = RATCHET_USERS.lock().await;
    let mut devs_init = RATCHET_DEVICES.lock().await;
    let write_txn = db.begin_write()?;
    {
        write_txn.open_table(RATCHET_USERS_TABLE.unwrap())?;
        write_txn.open_table(RATCHET_DEVS_TABLE.unwrap())?;
    }
    write_txn.commit()?;
    let read_txn = db.begin_read()?;
    let table = read_txn.open_table(RATCHET_USERS_TABLE.unwrap())?;

    let table_iter = table.iter()?;
    table_iter.for_each(|tup| {
        let v = tup.expect("dont get this interface");
        let key = v.0.value();
        let val = v.1.value();
        println!("Processing {:#?}", key);
        let new_user: RatchetUserEntry = serde_json::from_str(&val).unwrap();
        println!("Got {:#?}", new_user);
        users_init.insert(key.to_string(), new_user);
    });

    let read_txn = db.begin_read()?;
    let table = read_txn.open_table(RATCHET_DEVS_TABLE.unwrap())?;

    let table_iter = table.iter()?;
    table_iter.for_each(|tup| {
        let v = tup.expect("dont get this interface");
        let key = v.0.value();
        let val = v.1.value();
        println!("Processing {:#?}", key);
        let new_dev: RatchetDevEntry = serde_json::from_str(&val).unwrap();
        println!("Got {:#?}", new_dev);
        devs_init.insert(key.to_string(), new_dev);
    });

    Ok(())
}

fn add_user_routes(app: Rocket<Build>) -> Rocket<Build> {
    app.mount("/", rocket::routes![rm_user])
        .mount("/", rocket::routes![edit_user])
        .mount("/", rocket::routes![add_user])
        .mount("/", rocket::routes![get_users])
}

fn add_dev_routes(app: Rocket<Build>) -> Rocket<Build> {
    app.mount("/", rocket::routes![rm_dev])
        .mount("/", rocket::routes![edit_dev])
        .mount("/", rocket::routes![add_dev])
        .mount("/", rocket::routes![get_devs])
}
