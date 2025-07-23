#[macro_use]
extern crate rocket;

use std::env;

use db::Db;
use dotenv::dotenv;
use models::{AuthUser, CORS, Deck, User};
use rocket::{http::Status, serde::json::Json};
use sqlx::{Row, postgres::PgRow};

use models::{LoginRequest, LoginResponse};
use utils::generate_token;

use crate::{
    models::{Card, NewDeckRequest, RegisterRequest},
    utils::verify_token,
};

mod db;
mod models;
mod utils;

#[post("/login", data = "<data>")]
async fn login(data: Json<LoginRequest>) -> Result<Json<LoginResponse>, Status> {
    let db = Db::connect()
        .await
        .map_err(|_| Status::InternalServerError)?;

    let user = sqlx::query("SELECT * FROM users WHERE username = $1")
        .bind(&data.username)
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
        .map_err(|_| Status::InternalServerError)?;

    let user = match user {
        Some(u) => u,
        None => return Err(Status::Unauthorized),
    };

    let is_valid =
        bcrypt::verify(&data.password, &user.password).map_err(|_| Status::InternalServerError)?;

    if !is_valid {
        return Err(Status::Unauthorized);
    }

    let secret = env::var("JWT_SECRET").expect("JWT_SECRET must be set");
    let token = generate_token(&user.id, &secret);

    Ok(Json(LoginResponse { token }))
}

#[post("/register", data = "<data>")]
async fn register(data: Json<RegisterRequest>) -> Result<Json<LoginResponse>, Status> {
    use uuid::Uuid;

    let db = Db::connect()
        .await
        .map_err(|_| Status::InternalServerError)?;

    if data.username.trim().is_empty() || data.password.trim().is_empty() {
        return Err(Status::BadRequest);
    }

    let existing_user =
        sqlx::query_scalar::<_, String>("SELECT username FROM users WHERE username = $1")
            .bind(&data.username)
            .fetch_optional(db.pool())
            .await
            .map_err(|_| Status::InternalServerError)?;

    if existing_user.is_some() {
        return Err(Status::Conflict);
    }

    let hashed_password = bcrypt::hash(&data.password, bcrypt::DEFAULT_COST)
        .map_err(|_| Status::InternalServerError)?;

    let user_id = Uuid::new_v4().to_string();

    let result = sqlx::query("INSERT INTO users (id, username, password) VALUES ($1, $2, $3)")
        .bind(&user_id)
        .bind(&data.username)
        .bind(&hashed_password)
        .execute(db.pool())
        .await;

    if let Err(e) = result {
        eprintln!("Failed to create user: {:?}", e);
        return Err(Status::InternalServerError);
    }

    let secret = env::var("JWT_SECRET").expect("JWT_SECRET must be set");
    let token = generate_token(&user_id, &secret);

    Ok(Json(LoginResponse { token }))
}

#[get("/deck")]
async fn deck(user: AuthUser) -> Result<Json<Vec<Deck>>, Status> {
    let db = Db::connect()
        .await
        .map_err(|_| Status::InternalServerError)?;

    let decks = sqlx::query("SELECT * FROM decks WHERE user_id = $1")
        .bind(&user.user_id)
        .map(|row: PgRow| {
            let id = row.try_get("id").unwrap();
            let name = row.try_get("name").unwrap();
            Deck { id, name }
        })
        .fetch_all(db.pool())
        .await
        .map_err(|_| Status::InternalServerError)?;

    Ok(Json(decks))
}

#[get("/deck/<deck_id>/card")]
async fn cards_deck(deck_id: String, user: AuthUser) -> Result<Json<Vec<Card>>, Status> {
    let db = Db::connect()
        .await
        .map_err(|_| Status::InternalServerError)?;

    let cards = sqlx::query("SELECT * FROM cards WHERE deck_id = $1")
        .bind(&deck_id)
        .map(|row: PgRow| {
            let id = row.try_get("id").unwrap();
            let front = row.try_get("front").unwrap();
            let back = row.try_get("back").unwrap();
            Card { id, front, back }
        })
        .fetch_all(db.pool())
        .await
        .map_err(|_| Status::InternalServerError)?;

    Ok(Json(cards))
}

#[post("/deck/new", data = "<data>")]
async fn deck_new(user: AuthUser, data: Json<NewDeckRequest>) -> Result<Json<Deck>, Status> {
    use uuid::Uuid;

    let db = Db::connect()
        .await
        .map_err(|_| Status::InternalServerError)?;

    let existing_deck = sqlx::query_scalar::<_, String>("SELECT name FROM decks WHERE name = $1")
        .bind(&data.name)
        .fetch_optional(db.pool())
        .await
        .map_err(|_| Status::InternalServerError)?;

    if existing_deck.is_some() {
        return Err(Status::Conflict);
    }

    let deck = Deck {
        id: Uuid::new_v4().to_string(),
        name: data.name.clone(),
    };

    let result = sqlx::query("INSERT INTO decks (id, name, user_id) VALUES ($1, $2, $3)")
        .bind(&deck.id)
        .bind(&deck.name)
        .bind(&user.user_id)
        .execute(db.pool())
        .await;

    if let Err(e) = result {
        eprintln!("Failed to create deck: {:?}", e);
        return Err(Status::InternalServerError);
    }

    Ok(Json(deck))
}

#[get("/deck/shared")]
async fn deck_shared(user: AuthUser) -> Result<Json<Vec<Deck>>, Status> {
    let db = Db::connect()
        .await
        .map_err(|_| Status::InternalServerError)?;

    let decks = sqlx::query("SELECT * FROM decks WHERE is_shared = true")
        .map(|row: PgRow| {
            let id = row.try_get("id").unwrap();
            let name = row.try_get("name").unwrap();
            Deck { id, name }
        })
        .fetch_all(db.pool())
        .await
        .map_err(|_| Status::InternalServerError)?;

    Ok(Json(decks))
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
        .mount(
            "/",
            routes![
                deck,
                deck_new,
                deck_shared,
                cards_deck,
                rows,
                login,
                register,
                all_options,
                protected
            ],
        )
        .launch()
        .await?;

    Ok(())
}
