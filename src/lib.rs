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

use diesel::r2d2::ConnectionManager;
use diesel::SqliteConnection;
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
