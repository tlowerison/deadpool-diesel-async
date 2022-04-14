use deadpool::managed;
use deadpool_diesel_async::Manager;
use diesel_async::AsyncPgConnection;
use std::ops::{Deref, DerefMut};
use std::sync::Arc;
use tokio::sync::OwnedMutexGuard;

type Pool = managed::Pool<Manager<AsyncPgConnection>>;

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
        println!("Connected to database successfully");

        Ok(Db(Arc::new(pool)))
    }
}

cfg_if! {
    if #[cfg(not(feature = "db_conn_wrapper"))] {
        pub type DbConn = OwnedMutexGuard<AsyncPgConnection>;
    } else {
        pub struct DbConn(pub OwnedMutexGuard<AsyncPgConnection>);

        impl From<OwnedMutexGuard<AsyncPgConnection>> for DbConn {
            fn from(async_pg_connection: OwnedMutexGuard<AsyncPgConnection>) -> Self {
                Self(async_pg_connection)
            }
        }

        impl Deref for DbConn {
            type Target = AsyncPgConnection;
            fn deref(&self) -> &Self::Target {
                self.0.deref()
            }
        }

        impl DerefMut for DbConn {
            fn deref_mut(&mut self) -> &mut Self::Target {
                self.0.deref_mut()
            }
        }
    }
}
