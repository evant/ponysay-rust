use std::process::Command;
use assert_cmd::prelude::*;
use predicates::prelude::*;

#[test]
fn pony_list() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME"))?;
    cmd.arg("-l");

    let mut python_ponysay = Command::new("ponysay");
    python_ponysay.arg("-l");
    
}