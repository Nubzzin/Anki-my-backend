#[macro_use]
extern crate rocket;

use db::Db;
use dotenv::dotenv;
use rocket::http::ext::IntoCollection;
use rocket::serde::{Deserialize, Serialize, json::Json};
use rocket::{
    Request, Response,
    fairing::{Fairing, Info, Kind},
    http::Header,
};
use sqlx::Row;
use sqlx::postgres::PgRow;

mod db;

#[derive(Debug, Serialize, Deserialize)]
struct User {
    id: String,
    name: String,
    password: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Deck {
    id: String,
    name: String,
    description: String,
    cards: Vec<Card>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Card {
    id: String,
    front: String,
    back: String,
}

pub struct CORS;

#[rocket::async_trait]
impl Fairing for CORS {
    fn info(&self) -> Info {
        Info {
            name: "Add CORS headers",
            kind: Kind::Response,
        }
    }

    async fn on_response<'r>(&self, _req: &'r Request<'_>, res: &mut Response<'r>) {
        res.set_header(Header::new("Access-Control-Allow-Origin", "*"));
        res.set_header(Header::new(
            "Access-Control-Allow-Methods",
            "POST, GET, PATCH, OPTIONS",
        ));
        res.set_header(Header::new("Access-Control-Allow-Headers", "*"));
    }
}

#[get("/deck")]
fn deck() -> Json<Vec<Deck>> {
    Json(vec![
        Deck {
            id: "1".into(),
            name: "Deck 1".into(),
            description: "Description for Deck 1".into(),
            cards: vec![
                Card {
                    id: "1".into(),
                    front: "Front 1".into(),
                    back: "Back 1".into(),
                },
                Card {
                    id: "2".into(),
                    front: "Front 2".into(),
                    back: "Back 2".into(),
                },
            ],
        },
        Deck {
            id: "2".into(),
            name: "Deck 2".into(),
            description: "Description for Deck 2".into(),
            cards: vec![
                Card {
                    id: "3".into(),
                    front: "Front 3".into(),
                    back: "Back 3".into(),
                },
                Card {
                    id: "4".into(),
                    front: "Front 4".into(),
                    back: "Back 4".into(),
                },
            ],
        },
        Deck {
            id: "3".into(),
            name: "Deck 3".into(),
            description: "Description for Deck 3".into(),
            cards: vec![
                Card {
                    id: "5".into(),
                    front: "Front 5".into(),
                    back: "Back 5".into(),
                },
                Card {
                    id: "6".into(),
                    front: "Front 6".into(),
                    back: "Back 6".into(),
                },
            ],
        },
    ])
}

#[get("/deck/<id>")]
fn deck_id(id: String) -> String {
    format!("Hello, deck {}!", id)
}

// Test
#[get("/rows")]
async fn rows() -> Json<Vec<User>> {
    let db = Db::connect().await.unwrap();
    let rows = sqlx::query("SELECT * FROM test")
        .map(|row: PgRow| {
            let id = row.try_get("id").unwrap();
            let name = row.try_get("name").unwrap();
            let password = row.try_get("password").unwrap();
            User { id, name, password }
        })
        .fetch_all(db.pool())
        .await
        .unwrap();

    Json(rows)
}

#[rocket::main]
async fn main() -> Result<(), Box<rocket::Error>> {
    dotenv().ok();

    let db = Db::connect().await.unwrap();

    let rows = sqlx::query("SELECT * FROM test")
        .fetch_all(db.pool())
        .await
        .unwrap();

    println!("Row: {:#?}", rows);

    let _rocket = rocket::build()
        .attach(CORS)
        .mount("/", routes![deck, deck_id, rows])
        .launch()
        .await?;

    Ok(())
}
