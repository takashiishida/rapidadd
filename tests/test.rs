// Note: these tests were written and verified on macOS.
// Please adjust error handling and assertions accordingly if you encounter platform-specific issues.

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;
use tempfile::tempdir;
use tempfile::TempDir;

struct TestEnvironment {
    temp_dir: TempDir,
    daily_path: PathBuf,
}

// Create a temporary directory and a config file for testing
fn setup_test_environment() -> TestEnvironment {
    let temp_dir = tempdir().unwrap();
    let config_dir_path = temp_dir.path().join(".config/rapidadd");
    fs::create_dir_all(&config_dir_path).unwrap();
    let config_file_path = config_dir_path.join("config.toml");
    let daily_path = temp_dir.path().to_path_buf();

    // Create the config file with test-specific settings
    let mut config = File::create(&config_file_path).unwrap();
    writeln!(
        config,
        r#"
daily_path = "{}"
file_extension = "md"
date_format = "%Y-%m-%d"
"#,
        daily_path.to_string_lossy()
    )
    .unwrap();

    TestEnvironment {
        temp_dir,
        daily_path,
    }
}

#[test]
// test that the CLI fails when no args are provided
fn test_cli_no_args() {
    let _env = setup_test_environment();
    let mut cmd = Command::cargo_bin("rapidadd").unwrap();
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("The following required arguments were not provided:"));
}

#[test]
// test appending an entry to an existing daily file
fn test_cli_append_entry() {
    let env = setup_test_environment();
    let current_date = chrono::Local::now().format("%Y-%m-%d").to_string();
    let test_file_path = env.daily_path.join(format!("{}.md", current_date));
    let test_entry = "Test entry on existing file";

    // Create the daily file to simulate an existing file condition
    std::fs::write(&test_file_path, "Initial content\n").unwrap();

    // Simulate running the application to append an entry
    let mut cmd = Command::cargo_bin("rapidadd").unwrap();
    cmd.env("HOME", env.temp_dir.path())
       .arg(test_entry)
       .assert()
       .success();

    // Verify the file contains the expected entry
    let contents = std::fs::read_to_string(test_file_path).expect("Failed to read the daily file");
    assert!(contents.contains(test_entry), "The file does not contain the expected text.");
}


#[test]
// test printing the content of an existing daily file
fn test_cli_print() {
    let env = setup_test_environment();
    let current_date = chrono::Local::now().format("%Y-%m-%d").to_string();
    let test_file_path = env.daily_path.join(format!("{}.md", current_date));

    // Write something to the test file
    let mut file = File::create(&test_file_path).unwrap();
    writeln!(file, "Test content for printing").unwrap();

    // Simulate running the application with the print argument
    let mut cmd = Command::cargo_bin("rapidadd").unwrap();
    cmd.env("HOME", env.temp_dir.path())
       .arg("--print")
       .assert()
       .success()
       .stdout(predicate::str::contains("Test content for printing"));
}

#[test]
// test handling the scenario when attempting to print a non-existing daily file
fn test_cli_print_non_existing_file_error() {
    let env = setup_test_environment();

    // Attempt to run the application with the print argument when the daily file does not exist
    let mut cmd = Command::cargo_bin("rapidadd").unwrap();
    cmd.env("HOME", env.temp_dir.path())
       .arg("--print")
       .assert()
       .failure() // Assuming the application exits with a failure status in this case
       .stderr(predicate::str::contains("No such file or directory").or(predicate::str::contains("Failed to read the daily file")));
}

#[test]
// test handling the scenario when attempting to append an entry to a non-existing daily file
fn test_cli_append_entry_non_existing_file_error() {
    let env = setup_test_environment();
    let current_date = chrono::Local::now().format("%Y-%m-%d").to_string();
    let test_file_path = env.daily_path.join(format!("{}.md", current_date));
    let test_entry = "Test entry on non-existing file";

    // Ensure the daily file does not exist initially
    assert!(!test_file_path.exists(), "Daily file should not exist before test runs");

    // Simulate running the application to append an entry to a non-existing file
    let mut cmd = Command::cargo_bin("rapidadd").unwrap();
    cmd.env("HOME", env.temp_dir.path())
       .arg(test_entry)
       .assert()
       .failure() // Expecting failure since the file should not be created
       .stderr(predicate::str::contains("No such file or directory")); // Updated to match actual error message

    // Verify the file still does not exist after attempting to append the entry
    assert!(!test_file_path.exists(), "Daily file should not be created by the application");
}
