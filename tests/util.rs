#![allow(dead_code)]
use assert_cmd::{Command, assert::Assert};
use tempfile::NamedTempFile;
use std::io::Write;

pub const NO_ERROR: &str = "";
pub const SUCCESS: i32 = 0;
pub const BUILD_ERROR: i32 = 65;

pub fn assert_tokenize(input: &str) -> Assert {
    let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
    write!(temp_file, "{input}").expect("Failed to write to temp file");

    let mut cmd = Command::cargo_bin("codecrafters-interpreter").expect("Binary not found");
    cmd.args(&["tokenize", temp_file.path().to_str().unwrap()]);

    cmd.assert()
}

pub fn run_tokenize(
    input: &str, 
    expected: &str, 
    expected_error: &str, 
    expected_code: i32) 
{
    let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
    write!(temp_file, "{input}").expect("Failed to write to temp file");

    let mut cmd = Command::cargo_bin("codecrafters-interpreter").expect("Binary not found");
    cmd.args(&["tokenize", temp_file.path().to_str().unwrap()]);

    let output = cmd.output().expect("Failed to run binary");

    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
    let exit_code = output.status.code().unwrap_or(-1);

    assert_eq!(stdout, expected);
    assert_eq!(stderr, expected_error);
    assert_eq!(exit_code, expected_code);
}
