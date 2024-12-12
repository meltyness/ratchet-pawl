#[macro_use] extern crate rocket;

use rocket::{form::Form, fs::{relative, FileServer}};
// If we wanted or needed to serve files manually, we'd use `NamedFile`. Always
// prefer to use `FileServer`!
mod manual {
    use std::path::{PathBuf, Path};
    use rocket::fs::NamedFile;

    #[rocket::get("/<path..>")]
    pub async fn static_files(path: PathBuf) -> Option<NamedFile> {
        let mut path = Path::new(super::relative!("pawl-js/build/")).join(path);
        if path.is_dir() {
            path.push("index.html");
        }

        NamedFile::open(path).await.ok()
    }
}

#[derive(FromForm, Debug)]
struct NewUser<'a> {
    username: &'a str,
    passhash: &'a str,
}

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

#[post("/adduser", format="multipart/form-data", data = "<newuser>")]
fn add_user(newuser: Form<NewUser>) { 
    println!("{:#?}", newuser);
}

#[get("/getusers")]
fn get_users<'a>() -> &'a str {
    println!("Nah, I'm just not gonna go");
    "<test string>!"
}


#[launch]
fn rocket() -> _ {
    rocket::build().mount("/hey", routes![index])
        .mount("/", rocket::routes![add_user])
        .mount("/", rocket::routes![get_users])
        .mount("/", FileServer::from(relative!("pawl-js/build/")))

}