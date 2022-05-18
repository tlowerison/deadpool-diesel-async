use deadpool::managed;
use deadpool_diesel_async::Manager;
use diesel_async::AsyncPgConnection;
use std::ops::{Deref, DerefMut};
use std::sync::Arc;

pub type DbConn = AsyncPgConnection;
type Pool = managed::Pool<Manager<DbConn>>;

#[derive(Clone)]
pub struct Db(pub Arc<Pool>);

impl Deref for Db {
    type Target = Arc<Pool>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Db {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Db {
    pub async fn new(database_url: String) -> Result<Db, Box<dyn std::error::Error>> {
        let manager = Manager::new(database_url);
        let pool: Pool = managed::Pool::builder(manager).build()?;

        let db_conn = pool.get().await?;
        db_conn.ping().await?;

        Ok(Db(Arc::new(pool)))
    }
}
