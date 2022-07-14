// use std::process::Command;
use assert_cmd::Command;

#[test] //attribute, tells rust to run this function when using testing
fn runs() {
    let mut cmd = Command::cargo_bin("hello").unwrap();
    cmd.assert().success().stdout("Hello, World!\n");

    // checks that the command is executed correctly
    // let res = cmd.output();
    // assert!(res.is_ok()); //assert! compares a value to true
}

//executables located in the current directory are not included in the Path variable. Due to security reasons.

#[test]
fn true_ok() {
    let mut cmd = Command::cargo_bin("true").unwrap();
    cmd.assert().success();
}

#[test]
fn false_ok() {
    let mut cmd = Command::cargo_bin("false").unwrap();
    cmd.assert().failure(); //needed since false returns an eror exit code
}
