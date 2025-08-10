use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::process::Command;

#[test]
fn test_add_habit() {
    let mut cmd = Command::cargo_bin("my_cli").unwrap();
    cmd.arg("add").arg("--name").arg("test-habit");
    cmd.assert().success().stdout(predicate::str::contains("Habit 'test-habit' added."));
}

#[test]
fn test_list_habits() {
    let mut cmd = Command::cargo_bin("my_cli").unwrap();
    cmd.arg("add").arg("--name").arg("test-habit-for-list");
    cmd.assert().success();

    let mut cmd = Command::cargo_bin("my_cli").unwrap();
    cmd.arg("list");
    cmd.assert().success().stdout(predicate::str::contains("test-habit-for-list"));
}

#[test]
fn test_complete_habit() {
    let mut cmd = Command::cargo_bin("my_cli").unwrap();
    cmd.arg("add").arg("--name").arg("test-habit-for-complete");
    cmd.assert().success();

    let mut cmd = Command::cargo_bin("my_cli").unwrap();
    cmd.arg("complete").arg("test-habit-for-complete");
    cmd.assert().success().stdout(predicate::str::contains("Habit 'test-habit-for-complete' marked as complete"));
}

#[test]
fn test_streak() {
    let mut cmd = Command::cargo_bin("my_cli").unwrap();
    cmd.arg("add").arg("--name").arg("test-habit-for-streak");
    cmd.assert().success();

    let mut cmd = Command::cargo_bin("my_cli").unwrap();
    cmd.arg("complete").arg("test-habit-for-streak");
    cmd.assert().success();

    let mut cmd = Command::cargo_bin("my_cli").unwrap();
    cmd.arg("streak").arg("test-habit-for-streak");
    cmd.assert().success().stdout(predicate::str::contains("Current streak for 'test-habit-for-streak': 1 days"));
}

#[test]
fn test_delete_habit() {
    let mut cmd = Command::cargo_bin("my_cli").unwrap();
    cmd.arg("add").arg("--name").arg("test-habit-for-delete");
    cmd.assert().success();

    let mut cmd = Command::cargo_bin("my_cli").unwrap();
    cmd.arg("delete").arg("test-habit-for-delete");
    cmd.assert().success().stdout(predicate::str::contains("Habit 'test-habit-for-delete' and its history have been deleted."));
}
