use assert_cmd::Command;

use std::env::current_dir;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

fn assert_test_input(case: String) {
    let test_dir = current_dir()
        .unwrap()
        .join("tests")
        .join("test_cases")
        .join(Path::new(&case));

    let in_file = test_dir.join("in");
    let in_file = in_file.with_extension("csv");

    let out_file = test_dir.join("out");
    let out_file = out_file.with_extension("csv");

    let mut file = File::open(out_file).unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();

    let mut cli_comand = Command::cargo_bin("transactinator").unwrap();

    cli_comand
        .args(&[in_file.to_str().unwrap()])
        .assert()
        .success()
        .stdout(contents);
}

#[test]
fn test_should_test_hello_world() {
    assert_test_input("simple".to_string());
}

#[test]
fn test_duplicated_transactions() {
    assert_test_input("duplicated".to_string());
}

#[test]
fn test_no_money_available() {
    assert_test_input("no_money".to_string());
}

#[test]
fn test_precision_calculation() {
    assert_test_input("precision".to_string());
}

#[test]
fn test_open_dispute_for_wrong_ttype() {
    assert_test_input("dispute_wrong_ttype".to_string());
}

#[test]
fn test_open_dispute_for_different_client_but_same_tid() {
    assert_test_input("dispute_missing_tid".to_string());
}

#[test]
fn test_withdraw_while_transaction_in_dispute() {
    assert_test_input("withdraw_while_dispute".to_string());
}

#[test]
fn test_resolve_while_not_open_dispute() {
    assert_test_input("resolve_missing".to_string());
}

#[test]
fn test_resolve_open() {
    assert_test_input("resolve_open".to_string());
}

#[test]
fn test_chargeback_missing_dispute() {
    assert_test_input("chargeback_missing".to_string());
}

#[test]
fn test_chargeback_lock() {
    assert_test_input("chargeback_lock".to_string());
}
