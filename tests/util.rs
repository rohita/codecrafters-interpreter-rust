use assert_cmd::{Command, assert::Assert};
use tempfile::NamedTempFile;
use std::io::Write;

pub fn run_tokenize(input: &str) -> Assert {
    let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
    write!(temp_file, "{input}").expect("Failed to write to temp file");

    let mut cmd = Command::cargo_bin("codecrafters-interpreter").expect("Binary not found");
    cmd.args(&["tokenize", temp_file.path().to_str().unwrap()]);

    cmd.assert()
}
