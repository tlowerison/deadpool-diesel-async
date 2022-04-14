use deadpool::async_trait;
use deadpool::managed::{self, RecycleError, RecycleResult};
use diesel::dsl::sql_query;
use diesel_async::{AsyncConnection, RunQueryDsl};
use std::{fmt, future::Future, marker::PhantomData, sync::Arc};
use thiserror::Error;
use tokio::sync::Mutex;

#[derive(Debug, Error)]
pub enum Error {
    #[error("connection error: {0}")]
    Connection(#[from] diesel::result::ConnectionError),
    #[error("ping error: {0}")]
    Ping(#[from] diesel::result::Error),
}

#[derive(Clone, Debug)]
pub struct AsyncDieselConnection<C>
where
    C: Send + 'static,
{
    arc_mutex: Arc<Mutex<C>>,
}

impl<C> From<C> for AsyncDieselConnection<C>
where
    C: Send + 'static,
{
    fn from(c: C) -> Self {
        Self {
            arc_mutex: Arc::new(Mutex::new(c)),
        }
    }
}

impl<C> AsyncDieselConnection<C>
where
    C: AsyncConnection + Send + 'static,
    <C as AsyncConnection>::Backend: diesel::backend::DieselReserveSpecialization,
{
    pub async fn interact<F, FU, T, U>(&self, f: F) -> T
    where
        F: FnOnce(U) -> FU + Send + 'static,
        FU: Future<Output = T>,
        T: Send + 'static,
        U: From<tokio::sync::OwnedMutexGuard<C>>,
    {
        let guard = self.arc_mutex.clone().lock_owned().await;
        f(guard.into()).await
    }

    pub async fn ping(&self) -> Result<(), diesel::result::Error> {
        let arc_mutex = self.arc_mutex.clone();
        let mut guard = arc_mutex.lock().await;
        sql_query("SELECT 1")
            .execute(&mut *guard)
            .await
            .map(|_| ())
    }
}

/// [`Connection`] [`Manager`] for use with [`diesel_async`].
///
/// See the [`deadpool` documentation](deadpool) for usage examples.
///
/// [`Manager`]: managed::Manager
pub struct Manager<C: AsyncConnection> {
    database_url: String,
    _marker: PhantomData<fn() -> C>,
}

// Implemented manually to avoid unnecessary trait bound on `C` type parameter.
impl<C: AsyncConnection> fmt::Debug for Manager<C> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Manager")
            .field("database_url", &self.database_url)
            .field("_marker", &self._marker)
            .finish()
    }
}

impl<C: AsyncConnection> Manager<C> {
    /// Creates a new [`Manager`] which establishes [`Connection`]s to the given
    /// `database_url`.
    #[must_use]
    pub fn new<S: Into<String>>(database_url: S) -> Self {
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
    <C as AsyncConnection>::Backend: diesel::backend::DieselReserveSpecialization,
{
    type Type = AsyncDieselConnection<C>;
    type Error = Error;

    async fn create(&self) -> Result<Self::Type, Self::Error> {
        Ok(AsyncDieselConnection::from(C::establish(&self.database_url).await.map_err(Error::from)?))
    }

    async fn recycle(&self, async_diesel_conn: &mut Self::Type) -> RecycleResult<Self::Error> {
        async_diesel_conn
            .ping()
            .await
            .map(|_| ())
            .map_err(Error::Ping)
            .map_err(|err| RecycleError::Message(format!("{err}")))
    }
}
