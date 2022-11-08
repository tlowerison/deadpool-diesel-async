mod db;

use db::{Db, DbConn};
use diesel::dsl::sql_query;
use diesel_async::RunQueryDsl;
use std::borrow::BorrowMut;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let database_url = std::env::var("DATABASE_URL")?;
    let db = Db::new(database_url).await?;

    let mut conn = db.get().await?;
    let one = query(conn.borrow_mut()).await?;
    assert!(one == 1);

    Ok(())
}

async fn query(db_conn: &mut DbConn) -> Result<usize, diesel::result::Error> {
    sql_query("select 1").execute(db_conn).await
}
