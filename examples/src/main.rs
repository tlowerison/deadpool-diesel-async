#[macro_use]
extern crate cfg_if;

mod db;

use db::{Db, DbConn};
use diesel::dsl::sql_query;
use diesel_async::RunQueryDsl;
use std::ops::DerefMut;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let database_url = std::env::var("DATABASE_URL")?;
    let db = Db::new(database_url).await?;

    let conn = db.get().await?;
    let one = conn.interact(query).await?;
    println!("{one} == 1");

    Ok(())
}

async fn query(mut db_conn: DbConn) -> Result<usize, diesel::result::Error> {
    sql_query("select 1")
        .execute(db_conn.deref_mut())
        .await
}
