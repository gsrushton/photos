use diesel::{
    r2d2::{ConnectionManager, Pool, PoolError, PooledConnection},
    result::Error as DieselError,
    SqliteConnection,
};

pub mod model;
pub mod schema;

pub type Connection = PooledConnection<ConnectionManager<SqliteConnection>>;

pub type ConnectionPool = Pool<ConnectionManager<SqliteConnection>>;

#[derive(Clone)]
pub struct Guard(std::sync::Arc<std::sync::Mutex<()>>);

impl Guard {
    pub fn new() -> Self {
        Self(std::sync::Arc::new(std::sync::Mutex::new(())))
    }
}

impl std::ops::Deref for Guard {
    type Target = std::sync::Mutex<()>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, thiserror::Error)]
pub enum NewSystemError {
    #[error("Failed to connect to database")]
    DatabaseConnectionError(#[from] diesel::r2d2::PoolError),
    #[error("Failed to perform database migration")]
    DatabaseMigrationFailed(#[from] diesel_migrations::RunMigrationsError),
}

#[derive(Clone)]
pub struct System {
    connection_pool: ConnectionPool,
    appearances_guard: Guard,
    avatars_guard: Guard,
    people_guard: Guard,
    photos_guard: Guard,
}

impl System {
    pub fn new(db_file_path: &std::path::Path) -> Result<Self, NewSystemError> {
        #[derive(Debug)]
        struct ConnectionCustomiser;

        impl diesel::r2d2::CustomizeConnection<diesel::SqliteConnection, diesel::r2d2::Error>
            for ConnectionCustomiser
        {
            fn on_acquire(
                &self,
                conn: &mut diesel::SqliteConnection,
            ) -> Result<(), diesel::r2d2::Error> {
                use diesel::connection::SimpleConnection;
                conn.batch_execute("PRAGMA busy_timeout = 2000;")
                    .and_then(|_| conn.batch_execute("PRAGMA journal_mode = WAL;"))
                    .and_then(|_| conn.batch_execute("PRAGMA synchronous = NORMAL;"))
                    .and_then(|_| conn.batch_execute("PRAGMA foreign_keys = ON;"))
                    .map_err(diesel::r2d2::Error::QueryError)
            }
        }

        let connection_pool = diesel::r2d2::Pool::builder()
            .connection_customizer(Box::new(ConnectionCustomiser))
            .build(
                diesel::r2d2::ConnectionManager::<diesel::SqliteConnection>::new(
                    db_file_path.to_string_lossy(),
                ),
            )?;

        crate::embedded_migrations::run(&connection_pool.get()?)?;

        Ok(Self {
            connection_pool,
            appearances_guard: Guard::new(),
            avatars_guard: Guard::new(),
            people_guard: Guard::new(),
            photos_guard: Guard::new(),
        })
    }

    pub fn appearances_insertion_guard(&self) -> &Guard {
        &self.appearances_guard
    }

    pub fn avatars_insertion_guard(&self) -> &Guard {
        &self.avatars_guard
    }

    pub fn people_insertion_guard(&self) -> &Guard {
        &self.people_guard
    }

    pub fn photos_insertion_guard(&self) -> &Guard {
        &self.photos_guard
    }

    pub async fn run_query<F, T>(&self, f: F) -> Result<T, QueryError>
    where
        F: FnOnce(Connection) -> Result<T, DieselError> + Send + 'static,
        T: Send + 'static,
    {
        use actix_web::error::BlockingError;

        let connection = self.connection_pool.get()?;

        actix_web::web::block(move || f(connection))
            .await
            .map_err(|err| match err {
                BlockingError::Error(err) => QueryError::QueryError(err),
                BlockingError::Canceled => QueryError::OperationCancelled,
            })
    }
}

#[derive(Debug, thiserror::Error)]
pub enum QueryError {
    #[error("Failed to connect to the database")]
    ConnectionError(#[from] PoolError),
    #[error(transparent)]
    QueryError(DieselError),
    #[error("Operation cancelled")]
    OperationCancelled,
}

#[derive(Debug, thiserror::Error)]
pub enum UpdateQueryError {
    #[error(transparent)]
    QueryError(QueryError),
    #[error("No matching record was found")]
    NoSuchRecord,
}
