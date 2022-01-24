use std::fmt;

use diesel_derive_enum::DbEnum;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, DbEnum)]
pub enum Direction {
  North,
  East,
  South,
  West,
}

impl fmt::Display for Direction {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Direction::North => write!(f, "N"),
      Direction::East => write!(f, "E"),
      Direction::South => write!(f, "S"),
      Direction::West => write!(f, "W"),
    }
  }
}
