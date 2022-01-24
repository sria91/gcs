use std::fmt;
use std::str::FromStr;

use crate::{Direction, Error};

#[derive(Debug)]
pub struct Pose {
  x: i32,
  y: i32,
  facing: Direction,
}

impl Pose {
  pub fn new(x: i32, y: i32, facing: Direction) -> Self {
    Self { x, y, facing }
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
}

impl FromStr for Pose {
  type Err = Error;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    let pose: Vec<&str> = s.trim().split(' ').collect();

    match pose.len() {
      len if len != 3 => Err(Error::PoseParseError(format!("Expected 3 parameters but found {}", len))),
      _ => {
        let x: i32 = pose[0].parse()?;
        let y: i32 = pose[1].parse()?;
        let facing = match pose[2] {
          "N" => Direction::North,
          "E" => Direction::East,
          "S" => Direction::South,
          "W" => Direction::West,
          other => return Err(Error::PoseParseError(format!("Invalid direction '{}'", other))),
        };

        Ok(Self { x, y, facing })
      }
    }
  }
}

impl fmt::Display for Pose {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{} {} {}", self.x, self.y, self.facing)
  }
}
