use std::fmt;
use std::num::ParseIntError;

#[derive(Debug)]
pub enum Error {
  EmptyFile,
  InitialPoseNotFound,
  PathNotFound,
  CoOrdinateParseError(String),
  PoseParseError(String),
  MotionParseError(String),
}

impl fmt::Display for Error {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::EmptyFile => write!(f, "Error: Empty file"),
      Self::InitialPoseNotFound => write!(f, "Error: Initial pose for the rover not found"),
      Self::PathNotFound => write!(f, "Error: Path for the rover not found"),
      Self::CoOrdinateParseError(error) => write!(f, "Error parsing co-ordinate: {}", error),
      Self::PoseParseError(error) => write!(f, "Error parsing pose: {}", error),
      Self::MotionParseError(error) => write!(f, "Error parsing motion: {}", error),
    }
  }
}

impl From<ParseIntError> for Error {
  fn from(_: ParseIntError) -> Self {
    Self::CoOrdinateParseError("Co-ordinate can only be a non-negative integer".into())
  }
}

impl std::error::Error for Error {}
