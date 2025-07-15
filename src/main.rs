#[macro_use]
extern crate rocket;

use std::env;

use db::Db;
use dotenv::dotenv;
use models::{AuthUser, CORS, Deck, User};
use rocket::{http::Status, serde::json::Json};
use sqlx::{Row, postgres::PgRow};

use crate::{
    models::{LoginRequest, LoginResponse},
    utils::generate_token,
};

mod db;
mod models;
mod utils;

#[post("/login", data = "<data>")]
async fn login(data: Json<LoginRequest>) -> Json<LoginResponse> {
    let db = Db::connect().await.unwrap();
    let user = sqlx::query("SELECT * FROM users WHERE username = $1 AND password = $2")
        .bind(&data.username)
        .bind(&data.password)
        .map(|row: PgRow| {
            let id = row.try_get("id").unwrap();
            let username = row.try_get("username").unwrap();
            let password = row.try_get("password").unwrap();
            User {
                id,
                username,
                password,
            }
        })
        .fetch_optional(db.pool())
        .await
        .unwrap();

    let secret = env::var("JWT_SECRET").expect("JWT_SECRET must be set");

    let user_id = match user {
        Some(user) => user.id,
        None => return Json(LoginResponse { token: "".into() }),
    };
    let token = generate_token(user_id.as_ref(), secret.as_ref());

    Json(LoginResponse { token })
}

#[get("/deck")]
async fn deck() -> Json<Vec<Deck>> {
    let db = Db::connect().await.unwrap();
    let decks = sqlx::query("SELECT * FROM decks" /* WHERE user_id = $1*/)
        // .bind("a81bc81b-dead-4e5d-abff-90865d1e13b1") // Exmple
        .map(|row: PgRow| {
            let id = row.try_get("id").unwrap();
            let name = row.try_get("name").unwrap();
            Deck { id, name }
        })
        .fetch_all(db.pool())
        .await
        .expect("Failed to fetch decks");

    Json(decks)
}

// #[get("/deck/<id>")]
// async fn deck_id(id: String) -> Json<Deck> {
//     Json(Deck {
//         id: id.clone(),
//         name: format!("Deck {}", id),
//         description: format!("Description for Deck {}", id),
//         cards: vec![
//             Card {
//                 id: "1".into(),
//                 front: format!("Front 1 for Deck {}", id),
//                 back: format!("Back 1 for Deck {}", id),
//             },
//             Card {
//                 id: "2".into(),
//                 front: format!("Front 2 for Deck {}", id),
//                 back: format!("Back 2 for Deck {}", id),
//             },
//         ],
//     })
// }

// Test
#[get("/rows")]
async fn rows() -> Json<Vec<User>> {
    let db = Db::connect().await.unwrap();
    let rows = sqlx::query("SELECT * FROM users")
        .map(|row: PgRow| {
            let id = row.try_get("id").unwrap();
            let username = row.try_get("username").unwrap();
            let password = row.try_get("password").unwrap();
            User {
                id,
                username,
                password,
            }
        })
        .fetch_all(db.pool())
        .await
        .unwrap();

    Json(rows)
}

#[get("/protected")]
fn protected(user: Option<AuthUser>) -> Result<String, Status> {
    match user {
        Some(u) => Ok(format!("Hello, user with id: {}", u.user_id)),
        None => Err(Status::Unauthorized),
    }
}

#[options("/<_..>")]
fn all_options() -> &'static str {
    ""
}

#[rocket::main]
async fn main() -> Result<(), Box<rocket::Error>> {
    dotenv().ok();

    // let db = Db::connect().await.unwrap();

    // let rows = sqlx::query("SELECT * FROM test")
    //     .fetch_all(db.pool())
    //     .await
    //     .unwrap();

    // println!("Row: {:#?}", rows);

    let _rocket = rocket::build()
        .attach(CORS)
        .mount("/", routes![deck, rows, login, all_options, protected])
        .launch()
        .await?;

    Ok(())
}
