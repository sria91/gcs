#[macro_use]
extern crate actix_web;
#[macro_use]
extern crate diesel;

mod direction;
mod error;
mod motion;
pub mod plateau;
mod pose;
pub mod rover;
mod schema;

use std::time::Duration;

use diesel::SqliteConnection;
use diesel::{connection::SimpleConnection, r2d2::ConnectionManager, r2d2::CustomizeConnection};
use r2d2::{Pool, PooledConnection};

pub use direction::Direction;
pub use error::Error;
pub use motion::Motion;
pub use plateau::Plateau;
pub use pose::Pose;
pub use rover::Rover;

pub type DBPool = Pool<ConnectionManager<SqliteConnection>>;
pub type DBPooledConnection = PooledConnection<ConnectionManager<SqliteConnection>>;

pub const APPLICATION_JSON: &str = "application/json";
pub const CONNECTION_POOL_ERROR: &str = "couldn't get DB connection from pool";

#[cfg(debug_assertions)]
#[macro_export]
macro_rules! dprintln {
  ($($arg:tt)*) => ({
      println!("{}", format_args!($($arg)*));
  })
}

#[cfg(not(debug_assertions))]
#[macro_export]
macro_rules! dprintln {
  ($($arg:tt)*) => {};
}

#[derive(Debug)]
pub struct ConnectionOptions {
  pub enable_wal: bool,
  pub enable_foreign_keys: bool,
  pub busy_timeout: Option<Duration>,
}

impl CustomizeConnection<SqliteConnection, diesel::r2d2::Error> for ConnectionOptions {
  fn on_acquire(&self, conn: &mut SqliteConnection) -> Result<(), diesel::r2d2::Error> {
    (|| {
      if self.enable_wal {
        conn.batch_execute("PRAGMA journal_mode = WAL; PRAGMA synchronous = NORMAL;")?;
      }
      if self.enable_foreign_keys {
        conn.batch_execute("PRAGMA foreign_keys = ON;")?;
      }
      if let Some(d) = self.busy_timeout {
        conn.batch_execute(&format!("PRAGMA busy_timeout = {};", d.as_millis()))?;
      }
      Ok(())
    })()
    .map_err(diesel::r2d2::Error::QueryError)
  }
}
