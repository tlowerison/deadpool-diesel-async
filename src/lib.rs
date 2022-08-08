use deadpool::async_trait;
use deadpool::managed::{self, RecycleError, RecycleResult};
use diesel::{backend::DieselReserveSpecialization, dsl::sql_query};
use diesel_async::{AsyncConnection, RunQueryDsl};
use std::{borrow::Cow, fmt, marker::PhantomData};
use std::ops::{Deref, DerefMut};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("connection error: {0}")]
    Connection(#[from] diesel::result::ConnectionError),
    #[error("ping error: {0}")]
    Ping(#[from] diesel::result::Error),
}

#[derive(Clone, Debug)]
pub struct Connection<C>(C);

impl<C> AsMut<C> for Connection<C> {
    fn as_mut(&mut self) -> &mut C {
        &mut self.0
    }
}

impl<C> AsRef<C> for Connection<C> {
    fn as_ref(&self) -> &C {
        &self.0
    }
}

impl<C> From<C> for Connection<C> {
    fn from(conn: C) -> Self {
        Self(conn)
    }
}

impl<C> Deref for Connection<C> {
    type Target = C;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<C> DerefMut for Connection<C> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<C> Connection<C>
where
    C: AsyncConnection + Send + 'static,
    <C as AsyncConnection>::Backend: DieselReserveSpecialization,
{
    pub async fn ping(&mut self) -> Result<(), diesel::result::Error> {
        sql_query("SELECT 1")
            .execute(&mut self.0)
            .await
            .map(|_| ())
    }
}

pub struct Manager<C> {
    database_url: Cow<'static, str>,
    _marker: PhantomData<fn() -> C>,
}

impl<C> fmt::Debug for Manager<C> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Manager")
            .field("database_url", &self.database_url)
            .field("_marker", &self._marker)
            .finish()
    }
}

impl<C> Manager<C> {
    #[must_use]
    pub fn new<S: Into<Cow<'static, str>>>(database_url: S) -> Self {
        Manager {
            database_url: database_url.into(),
            _marker: PhantomData,
        }
    }
}

#[async_trait]
impl<C> managed::Manager for Manager<C>
where
    C: AsyncConnection + 'static,
    <C as AsyncConnection>::Backend: DieselReserveSpecialization,
{
    type Type = Connection<C>;
    type Error = Error;

    async fn create(&self) -> Result<Self::Type, Self::Error> {
        Ok(Connection::from(C::establish(&self.database_url).await.map_err(Error::from)?))
    }

    async fn recycle(&self, connection: &mut Self::Type) -> RecycleResult<Self::Error> {
        connection
            .ping()
            .await
            .map(|_| ())
            .map_err(Error::Ping)
            .map_err(|err| RecycleError::Message(format!("{err}")))
    }
}
