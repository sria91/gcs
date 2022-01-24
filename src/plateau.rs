use std::{fmt, ops::Deref, str::FromStr};

use actix_web::{
  web::{self, Data, Json, Path},
  HttpResponse,
};
use chrono::{DateTime, NaiveDateTime, TimeZone, Utc};
use diesel::{ExpressionMethods, Insertable, QueryDsl, Queryable, RunQueryDsl};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{rover::Rovers, Error};
use crate::{
  rover::{RoverDB, RoverRequest},
  schema::plateaus,
  Motion, Rover,
};
use crate::{DBPool, DBPooledConnection};
use crate::{APPLICATION_JSON, CONNECTION_POOL_ERROR};

#[derive(Debug, Deserialize, Serialize)]
pub struct Plateaus(Vec<Plateau>);

impl Plateaus {
  pub fn new() -> Self {
    Self { 0: vec![] }
  }
}

impl Deref for Plateaus {
  type Target = Vec<Plateau>;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

#[derive(Debug, Clone, Deserialize, Serialize, AsExpression)]
pub struct Plateau {
  id: String,
  created_at: DateTime<Utc>,
  x_max: i32,
  y_max: i32,
}

impl Plateau {
  pub fn new(x_max: i32, y_max: i32) -> Self {
    Self {
      id: Uuid::new_v4().to_hyphenated().to_string(),
      created_at: Utc::now(),
      x_max,
      y_max,
    }
  }

  pub fn id(&self) -> &str {
    &self.id
  }

  pub fn x_max(&self) -> i32 {
    self.x_max
  }

  pub fn y_max(&self) -> i32 {
    self.y_max
  }

  fn to_plateau_db(&self) -> PlateauDB {
    PlateauDB {
      id: self.id.clone(),
      created_at: Utc::now().naive_utc(),
      x_max: self.x_max,
      y_max: self.y_max,
    }
  }
}

#[derive(Queryable, Insertable, AsChangeset)]
#[table_name = "plateaus"]
struct PlateauDB {
  id: String,
  created_at: NaiveDateTime,
  x_max: i32,
  y_max: i32,
}

impl PlateauDB {
  fn to_plateau(&self) -> Plateau {
    Plateau {
      id: self.id.clone(),
      created_at: Utc.from_utc_datetime(&self.created_at),
      x_max: self.x_max,
      y_max: self.y_max,
    }
  }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PlateauRequest {
  pub x_max: i32,
  pub y_max: i32,
}

impl PlateauRequest {
  fn to_plateau(&self) -> Plateau {
    Plateau::new(self.x_max, self.y_max)
  }
}

pub fn get_plateau(_plateau_id: Uuid, conn: &DBPooledConnection) -> Result<Plateau, diesel::result::Error> {
  use crate::schema::plateaus::dsl::*;

  let res = plateaus.filter(id.eq(_plateau_id.to_string())).load::<PlateauDB>(conn);
  match res {
    Ok(plateaus_db) => match plateaus_db.first() {
      Some(plateau_db) => Ok(plateau_db.to_plateau()),
      _ => Err(diesel::result::Error::NotFound),
    },
    Err(err) => Err(err),
  }
}

#[get("/plateaus/{plateau_id}")]
pub async fn async_get(path: Path<(String,)>, pool: Data<DBPool>) -> HttpResponse {
  let conn = pool.get().expect(CONNECTION_POOL_ERROR);
  let (id,) = path.0;
  let plateau = web::block(move || get_plateau(Uuid::from_str(&id).unwrap(), &conn)).await;

  match plateau {
    Ok(plateau) => HttpResponse::Ok().content_type(APPLICATION_JSON).json(plateau),
    _ => HttpResponse::NoContent().content_type(APPLICATION_JSON).await.unwrap(),
  }
}

pub fn list_plateaus(total_plateaus: i64, conn: &DBPooledConnection) -> Result<Plateaus, diesel::result::Error> {
  use crate::schema::plateaus::dsl::*;

  let _plateaus = match plateaus.order(created_at.desc()).limit(total_plateaus).load::<PlateauDB>(conn) {
    Ok(_plateaus) => _plateaus,
    Err(_) => vec![],
  };

  Ok(Plateaus {
    0: _plateaus.into_iter().map(|r| r.to_plateau()).collect::<Vec<Plateau>>(),
  })
}

#[get("/plateaus")]
pub async fn async_list(pool: Data<DBPool>) -> HttpResponse {
  let conn = pool.get().expect(CONNECTION_POOL_ERROR);
  let plateaus = web::block(move || list_plateaus(50, &conn)).await.unwrap();

  HttpResponse::Ok().content_type(APPLICATION_JSON).json(plateaus)
}

pub fn create_plateau(plateau: Plateau, conn: &DBPooledConnection) -> Result<Plateau, diesel::result::Error> {
  use crate::schema::plateaus::dsl::*;

  let plateau_db = plateau.to_plateau_db();
  let _ = diesel::insert_into(plateaus).values(&plateau_db).execute(conn);

  Ok(plateau_db.to_plateau())
}

#[post("/plateaus")]
pub async fn async_create(plateau_request: Json<PlateauRequest>, pool: Data<DBPool>) -> HttpResponse {
  let conn = pool.get().expect(CONNECTION_POOL_ERROR);

  let plateau = web::block(move || create_plateau(plateau_request.to_plateau(), &conn)).await;

  match plateau {
    Ok(plateau) => HttpResponse::Created().content_type(APPLICATION_JSON).json(plateau),
    _ => HttpResponse::NoContent().await.unwrap(),
  }
}

impl FromStr for Plateau {
  type Err = Error;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    let co_ordinates: Vec<&str> = s.trim().split(' ').collect();

    match co_ordinates.len() {
      len if len < 2 => Err(Error::CoOrdinateParseError("Y co-ordinate may be missing".into())),
      len if len > 2 => Err(Error::CoOrdinateParseError("Too many co-ordinates".into())),
      _ => {
        let x_max: i32 = co_ordinates[0].parse()?;
        let y_max: i32 = co_ordinates[1].parse()?;

        Ok(Self {
          id: Uuid::new_v4().to_hyphenated().to_string(),
          created_at: Utc::now(),
          x_max,
          y_max,
        })
      }
    }
  }
}

impl fmt::Display for Plateau {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{} {}", self.x_max, self.y_max)
  }
}

impl fmt::Display for Plateaus {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    writeln!(f, "{{")?;
    for plateau in self.iter() {
      writeln!(f, "  {}: {} {}", plateau.id, plateau.x_max, plateau.y_max)?;
    }
    write!(f, "}}")
  }
}

pub fn move_rover(_plateau_id: Uuid, _rover_id: Uuid, motion_vector: Vec<Motion>, conn: &DBPooledConnection) -> Result<Rover, diesel::result::Error> {
  let rover = get_rover(_plateau_id, _rover_id, conn);
  let plateau = get_plateau(_plateau_id, conn);
  match plateau {
    Ok(plateau) => match rover {
      Ok(mut rover) => {
        rover.apply_motion_vector(motion_vector, &plateau);
        use crate::schema::rovers::dsl::*;
        let _ = diesel::update(rovers.find(_rover_id.to_string()))
          .set(rover.to_rover_db(_plateau_id.to_string()))
          .execute(conn);
        Ok(rover)
      }
      Err(err) => Err(err),
    },
    Err(err) => Err(err),
  }
}

#[patch("/plateaus/{plateau_id}/rovers/{rover_id}/{motion}")]
pub async fn async_move_rover(path: Path<(String, String, String)>, pool: Data<DBPool>) -> HttpResponse {
  let conn = pool.get().expect(CONNECTION_POOL_ERROR);
  let (plateau_id, rover_id, motion_path) = path.0;
  let motion_vector = Motion::parse_path(&motion_path);
  match motion_vector {
    Ok(motion_vector) => {
      let rover = web::block(move || move_rover(Uuid::from_str(&plateau_id).unwrap(), Uuid::from_str(&rover_id).unwrap(), motion_vector, &conn)).await;
      match rover {
        Ok(rover) => HttpResponse::Ok().content_type(APPLICATION_JSON).json(rover),
        _ => HttpResponse::NotFound().content_type(APPLICATION_JSON).await.unwrap(),
      }
    }
    _ => HttpResponse::BadRequest().content_type(APPLICATION_JSON).await.unwrap(),
  }
}

pub fn get_rover_n(n: i64, _plateau_id: Uuid, conn: &DBPooledConnection) -> Result<Rover, diesel::result::Error> {
  use crate::schema::rovers::dsl::*;
  let res = rovers
    .filter(plateau_id.eq(_plateau_id.to_string()))
    .order(created_at.desc())
    .offset(n)
    .limit(1)
    .load::<RoverDB>(conn);
  match res {
    Ok(rovers_db) => match rovers_db.first() {
      Some(rover_db) => Ok(rover_db.to_rover()),
      _ => Err(diesel::result::Error::NotFound),
    },
    Err(err) => Err(err),
  }
}

pub fn get_rovers_count(_plateau_id: Uuid, conn: &DBPooledConnection) -> Result<usize, diesel::result::Error> {
  use crate::schema::rovers::dsl::*;
  rovers.count().execute(conn)
}

pub fn get_rover(_plateau_id: Uuid, _rover_id: Uuid, conn: &DBPooledConnection) -> Result<Rover, diesel::result::Error> {
  use crate::schema::rovers::dsl::*;

  let res = rovers
    .filter(plateau_id.eq(_plateau_id.to_string()))
    .filter(id.eq(_rover_id.to_string()))
    .load::<RoverDB>(conn);
  match res {
    Ok(rovers_db) => match rovers_db.first() {
      Some(rover_db) => Ok(rover_db.to_rover()),
      _ => Err(diesel::result::Error::NotFound),
    },
    Err(err) => Err(err),
  }
}

#[get("/plateaus/{plateau_id}/rovers/{rover_id}")]
pub async fn async_get_rover(path: Path<(String, String)>, pool: Data<DBPool>) -> HttpResponse {
  let conn = pool.get().expect(CONNECTION_POOL_ERROR);
  let (plateau_id, rover_id) = path.0;
  let rover = web::block(move || get_rover(Uuid::from_str(&plateau_id).unwrap(), Uuid::from_str(&rover_id).unwrap(), &conn)).await;

  match rover {
    Ok(rover) => HttpResponse::Ok().content_type(APPLICATION_JSON).json(rover),
    _ => HttpResponse::NoContent().content_type(APPLICATION_JSON).await.unwrap(),
  }
}

pub fn list_rovers(_plateau_id: Uuid, total_rovers: i64, conn: &DBPooledConnection) -> Result<Rovers, diesel::result::Error> {
  use crate::schema::rovers::dsl::*;

  let _rovers = match rovers
    .filter(plateau_id.eq(_plateau_id.to_string()))
    .order(created_at.desc())
    .limit(total_rovers)
    .load::<RoverDB>(conn)
  {
    Ok(_rovers) => _rovers,
    Err(_) => vec![],
  };

  Ok(Rovers::new(_rovers.into_iter().map(|r| r.to_rover()).collect::<Vec<Rover>>()))
}

#[get("/plateaus/{plateau_id}/rovers")]
pub async fn async_list_rovers(path: Path<(String,)>, pool: Data<DBPool>) -> HttpResponse {
  let conn = pool.get().expect(CONNECTION_POOL_ERROR);
  let (plateau_id,) = path.0;
  let rovers = web::block(move || list_rovers(Uuid::from_str(&plateau_id).unwrap(), 50, &conn)).await.unwrap();

  HttpResponse::Ok().content_type(APPLICATION_JSON).json(rovers)
}

pub fn create_rover(_plateau_id: Uuid, rover: Rover, conn: &DBPooledConnection) -> Result<Rover, diesel::result::Error> {
  use crate::schema::rovers::dsl::*;

  let rover_db = rover.to_rover_db(_plateau_id.to_string());
  let _ = diesel::insert_into(rovers).values(&rover_db).execute(conn);

  Ok(rover_db.to_rover())
}

#[post("/plateaus/{plateau_id}/rovers")]
pub async fn async_create_rover(path: Path<(String,)>, rover_request: Json<RoverRequest>, pool: Data<DBPool>) -> HttpResponse {
  let conn = pool.get().expect(CONNECTION_POOL_ERROR);

  let (plateau_id,) = path.0;
  let rover = web::block(move || create_rover(Uuid::from_str(&plateau_id).unwrap(), rover_request.to_rover(), &conn)).await;

  match rover {
    Ok(rover) => HttpResponse::Created().content_type(APPLICATION_JSON).json(rover),
    _ => HttpResponse::NoContent().await.unwrap(),
  }
}
