use std::{env, io};

use actix_web::{middleware, App, HttpServer};
use diesel::{r2d2::ConnectionManager, SqliteConnection};
use dotenv::dotenv;

use gcs::*;

#[actix_rt::main]
async fn main() -> Result<(), io::Error> {
  dotenv().ok();
  env_logger::init();

  let database_url = env::var("DATABASE_URL").expect("DATABASE_URL");
  let manager = ConnectionManager::<SqliteConnection>::new(database_url);
  let pool = r2d2::Pool::builder().build(manager).expect("Failed to create pool");

  HttpServer::new(move || {
    App::new()
      // Set up DB pool to be used with web::Data<Pool> extractor
      .data(pool.clone())
      // enable logger - always register actix-web Logger middleware last
      .wrap(middleware::Logger::default())
      // register HTTP requests handlers
      .service(plateau::async_create)
      .service(plateau::async_list)
      .service(plateau::async_get)
      .service(plateau::async_create_rover)
      .service(plateau::async_list_rovers)
      .service(plateau::async_get_rover)
      .service(plateau::async_move_rover)
  })
  .bind("0.0.0.0:9090")?
  .run()
  .await
}
