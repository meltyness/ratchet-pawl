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
use std::collections::HashMap;

use redb::{Database, ReadableTable, TableDefinition};

// TODO: What happens if we modify the user format by adding a field do we have to make it option type?
const RATCHET_USERS_TABLE: TableDefinition<&str, &str> = TableDefinition::new("ratchet_users");
const RATCHET_DEVS_TABLE: TableDefinition<&str, &str> = TableDefinition::new("ratchet_devs");


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

#[post("/rmuser", format = "multipart/form-data", data = "<username>")]
async fn rm_user(username: Form<String>) -> status::Custom<&'static str> {
    let mut users = RATCHET_USERS.lock().await;
    match users.remove(&*username) {
        Some(_) => {
            rtp_rm_user(username.clone()).await.expect("Database Error: ");
            status::Custom(Status::Ok, "")
        },
        None => status::Custom(Status::Gone, ""),
    }
}

#[post("/adduser", format = "multipart/form-data", data = "<newuser>")]
async fn add_user(newuser: Form<RatchetUserEntry>) -> status::Custom<&'static str> {
    let mut users = RATCHET_USERS.lock().await;
    if !users.contains_key(&newuser.username) {
        users.insert(
            newuser.username.clone(),
            RatchetUserEntry {
                username: newuser.username.clone(),
                passhash: newuser.passhash.clone(),
            },
        );
        rtp_write_user(
            RatchetUserEntry {
                username: newuser.username.clone(),
                passhash: newuser.passhash.clone(),
            },
        ).await.expect("Database Error: ");
        status::Custom(Status::Ok, "")
    } else {
        status::Custom(Status::Conflict, "")
    }
}

#[post("/edituser", format = "multipart/form-data", data = "<edited>")]
async fn edit_user(edited: Form<RatchetUserEntry>) -> status::Custom<&'static str> {
    let users = RATCHET_USERS.lock().await;
    if !users.contains_key(&edited.username) {
        status::Custom(Status::Gone, "")
    } else {
        rtp_write_user(
            RatchetUserEntry {
                username: edited.username.clone(),
                passhash: edited.passhash.clone(),
            },
        ).await.expect("Database Error");
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

#[derive(Clone, FromForm, Debug, Serialize)]
struct NewDev {
    network_id: String,
    key: String,
}

#[post("/rmdev", format = "multipart/form-data", data = "<network_id>")]
async fn rm_dev(network_id: Form<String>) -> status::Custom<&'static str> {
    let mut devs = RATCHET_DEVICES.lock().await;
    match devs.remove(&*network_id) {
        Some(_) => {
            rtp_rm_dev(network_id.clone()).await.expect("Database Error: ");
            status::Custom(Status::Ok, "")
        },
        None => status::Custom(Status::Gone, ""),
    }
}

#[post("/adddev", format = "multipart/form-data", data = "<newdev>")]
async fn add_dev(newdev: Form<NewDev>) -> status::Custom<&'static str> {
    let mut devs = RATCHET_DEVICES.lock().await;
    // TODO: Replace this with networkier stuff
    if !devs.contains_key(&newdev.network_id) {
        devs.insert(
            newdev.network_id.clone(),
            RatchetDevEntry {
                network_id: newdev.network_id.clone(),
                key: newdev.key.clone(),
            },
        );
        rtp_write_dev(
            RatchetDevEntry {
                network_id: newdev.network_id.clone(),
                key: newdev.key.clone(),
            },
        ).await.expect("Database Error: ");
        status::Custom(Status::Ok, "")
    } else {
        status::Custom(Status::Conflict, "")
    }
}

#[post("/editdev", format = "multipart/form-data", data = "<edited>")]
async fn edit_dev(edited: Form<NewDev>) -> status::Custom<&'static str> {
    let devs = RATCHET_DEVICES.lock().await;
    if !devs.contains_key(&edited.network_id) {
        status::Custom(Status::Gone, "")
    } else {
        rtp_write_dev(
            RatchetDevEntry {
                network_id: edited.network_id.clone(),
                key: edited.key.clone(),
            },
        ).await.expect("Database Error: ");
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
    let db = Database::create("ratchet_db.redb")?;
    let mut users_init = RATCHET_USERS.lock().await;
    let mut devs_init = RATCHET_DEVICES.lock().await;
    let write_txn = db.begin_write()?;
    {
        write_txn.open_table(RATCHET_USERS_TABLE)?;
        write_txn.open_table(RATCHET_DEVS_TABLE)?;
    }
    write_txn.commit()?;
    let read_txn = db.begin_read()?;
    let table = read_txn.open_table(RATCHET_USERS_TABLE)?;

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
    let table = read_txn.open_table(RATCHET_DEVS_TABLE)?;

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

async fn rtp_write_user(user: RatchetUserEntry) -> Result<(), redb::Error> {
    let db = Database::create("ratchet_db.redb")?;
    let write_txn = db.begin_write()?;
    {
        let mut table = write_txn.open_table(RATCHET_USERS_TABLE)?;
        let ser = serde_json::to_string(&user).unwrap();
        table.insert(&*user.username, &*ser)?;
    }
    write_txn.commit()?;
    Ok(())
}

async fn rtp_rm_user(usr: String) -> Result<(), redb::Error> {
    let db = Database::create("ratchet_db.redb")?;
    let write_txn = db.begin_write()?;
    {
        let mut table = write_txn.open_table(RATCHET_USERS_TABLE)?;
        println!("Removing {:?}", usr);
        table.remove(&*usr)?;
    }
    write_txn.commit()?;
    Ok(())
}

async fn rtp_write_dev(dev: RatchetDevEntry) -> Result<(), redb::Error> {
    let db = Database::create("ratchet_db.redb")?;
    let write_txn = db.begin_write()?;
    {
        let mut table = write_txn.open_table(RATCHET_DEVS_TABLE)?;
        let ser = serde_json::to_string(&dev).unwrap();
        table.insert(&*dev.network_id, &*ser)?;
    }
    write_txn.commit()?;
    Ok(())
}

async fn rtp_rm_dev(dev: String) -> Result<(), redb::Error> {
    let db = Database::create("ratchet_db.redb")?;
    let write_txn = db.begin_write()?;
    {
        let mut table = write_txn.open_table(RATCHET_DEVS_TABLE)?; // TODO: into the type system
        table.remove(&*dev)?;
    }
    write_txn.commit()?;
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
