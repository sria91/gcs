use std::fs::read_to_string;
use std::process::Command;

#[test]
fn test_stdout() {
  let output = if cfg!(target_os = "windows") {
    Command::new("cargo")
      .args(["run", "--release", "--", "--input", ".\\tests\\input.txt"])
      .output()
      .expect("failed to execute process")
  } else {
    Command::new("cargo")
      .args(["run", "--release", "--", "--input", "./tests/input.txt"])
      .output()
      .expect("failed to execute process")
  };
  let stdout = String::from_utf8(output.stdout).expect("Found invalid UTF-8");
  if cfg!(target_os = "windows") {
    assert_eq!(stdout, read_to_string(".\\tests\\output.txt").unwrap());
  } else {
    assert_eq!(stdout, read_to_string("./tests/output.txt").unwrap());
  }
}

#[test]
fn test_output() {
  if cfg!(target_os = "windows") {
    Command::new("cargo")
      .args(["run", "--release", "--", "--input", ".\\tests\\input.txt", "--output", "output.txt"])
      .output()
      .expect("failed to execute process");
  } else {
    Command::new("cargo")
      .args(["run", "--release", "--", "--input", "./tests/input.txt", "--output", "output.txt"])
      .output()
      .expect("failed to execute process");
  };
  if cfg!(target_os = "windows") {
    assert_eq!(read_to_string(".\\output.txt").unwrap(), read_to_string(".\\tests\\output.txt").unwrap());
  } else {
    assert_eq!(read_to_string("./output.txt").unwrap(), read_to_string("./tests/output.txt").unwrap());
  }
}
