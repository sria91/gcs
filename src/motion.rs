use serde::{Deserialize, Serialize};

use crate::Error;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum Motion {
  TurnLeft,
  TurnRight,
  MoveStraight,
}

impl Motion {
  pub fn parse_path(s: &str) -> Result<Vec<Self>, Error> {
    let mut path = Vec::new();
    for maybe_motion in s.chars() {
      match maybe_motion {
        'L' => path.push(Self::TurnLeft),
        'R' => path.push(Self::TurnRight),
        'M' => path.push(Self::MoveStraight),
        m => return Err(Error::MotionParseError(format!("Invalid motion '{}'", m))),
      }
    }
    Ok(path)
  }
}
