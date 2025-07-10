use std::env;

use sqlx::{
    Pool, Postgres,
    postgres::{PgPoolOptions, PgRow},
};

pub struct Db {
    pool: Pool<Postgres>,
}

impl Db {
    pub async fn connect() -> Result<Self, sqlx::Error> {
        let db_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(&db_url)
            .await?;

        Ok(Db { pool })
    }

    pub fn pool(&self) -> &Pool<Postgres> {
        &self.pool
    }
}
