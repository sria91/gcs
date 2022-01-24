use std::{fmt, ops::Deref};

use chrono::{DateTime, NaiveDateTime, TimeZone, Utc};
use diesel::{Insertable, Queryable};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::Plateau;
use crate::{direction::Direction, schema::rovers};
use crate::{motion::Motion, pose::Pose};

#[derive(Debug, Deserialize, Serialize)]
pub struct Rovers(Vec<Rover>);

impl Rovers {
  pub fn new(rovers: Vec<Rover>) -> Self {
    Self { 0: rovers }
  }
}

impl Deref for Rovers {
  type Target = Vec<Rover>;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Rover {
  id: String,
  created_at: DateTime<Utc>,
  x: i32,
  y: i32,
  facing: Direction,
}

impl Rover {
  pub fn new(x: i32, y: i32, facing: Direction) -> Self {
    Self {
      id: Uuid::new_v4().to_hyphenated().to_string(),
      created_at: Utc::now(),
      x,
      y,
      facing,
    }
  }

  pub fn id(&self) -> &str {
    &self.id
  }

  pub fn pose(&self) -> Pose {
    Pose::new(self.x, self.y, self.facing.clone())
  }

  pub fn x(&self) -> i32 {
    self.x
  }

  pub fn y(&self) -> i32 {
    self.y
  }

  pub fn facing(&self) -> Direction {
    self.facing.clone()
  }

  pub fn turn_left(&mut self) {
    match self.facing {
      Direction::North => {
        self.facing = Direction::West;
      }
      Direction::East => {
        self.facing = Direction::North;
      }
      Direction::South => {
        self.facing = Direction::East;
      }
      Direction::West => {
        self.facing = Direction::South;
      }
    }
  }

  pub fn turn_right(&mut self) {
    match self.facing {
      Direction::North => {
        self.facing = Direction::East;
      }
      Direction::East => {
        self.facing = Direction::South;
      }
      Direction::South => {
        self.facing = Direction::West;
      }
      Direction::West => {
        self.facing = Direction::North;
      }
    }
  }

  pub fn move_x_forward(&mut self, x_max: i32) {
    if self.x < x_max {
      self.x += 1;
    }
  }

  pub fn move_x_backward(&mut self) {
    if self.x > 0 {
      self.x -= 1;
    }
  }

  pub fn move_y_forward(&mut self, y_max: i32) {
    if self.y < y_max {
      self.y += 1;
    }
  }

  pub fn move_y_backward(&mut self) {
    if self.y > 0 {
      self.y -= 1;
    }
  }

  pub fn move_straight(&mut self, plateau: &Plateau) {
    match self.facing {
      Direction::North => self.move_y_forward(plateau.y_max()),
      Direction::East => self.move_x_forward(plateau.x_max()),
      Direction::South => self.move_y_backward(),
      Direction::West => self.move_x_backward(),
    };
  }

  pub fn apply_motion(&mut self, motion: Motion, plateau: &Plateau) {
    match motion {
      Motion::TurnLeft => self.turn_left(),
      Motion::TurnRight => self.turn_right(),
      Motion::MoveStraight => self.move_straight(plateau),
    };
  }

  pub fn apply_motion_vector(&mut self, motion_vector: Vec<Motion>, plateau: &Plateau) {
    for motion in motion_vector {
      self.apply_motion(motion, plateau)
    }
  }

  pub fn to_rover_db(&self, plateau_id: String) -> RoverDB {
    RoverDB {
      id: self.id.clone(),
      created_at: Utc::now().naive_utc(),
      x: self.x,
      y: self.y,
      facing: self.facing.clone(),
      plateau_id,
    }
  }
}

#[derive(Queryable, Insertable, AsChangeset)]
#[table_name = "rovers"]
pub struct RoverDB {
  id: String,
  created_at: NaiveDateTime,
  x: i32,
  y: i32,
  facing: Direction,
  plateau_id: String,
}

impl RoverDB {
  pub fn to_rover(&self) -> Rover {
    Rover {
      id: self.id.clone(),
      created_at: Utc.from_utc_datetime(&self.created_at),
      x: self.x,
      y: self.y,
      facing: self.facing.clone(),
    }
  }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RoverRequest {
  pub x: i32,
  pub y: i32,
  pub facing: Direction,
}

impl RoverRequest {
  pub fn to_rover(&self) -> Rover {
    Rover::new(self.x, self.y, self.facing.clone())
  }
}

impl fmt::Display for Rover {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "Rover {} is at {} {} {}", self.id, self.x, self.y, self.facing)
  }
}

impl fmt::Display for Rovers {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    writeln!(f, "{{")?;
    for rover in self.iter() {
      writeln!(f, "  {}: {} {} {}", rover.id, rover.x, rover.y, rover.facing)?;
    }
    write!(f, "}}")
  }
}
