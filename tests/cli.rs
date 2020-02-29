use std::process::Command;
use assert_cmd::cargo::CommandCargoExt;

#[test]
fn pony_list() -> Result<(), Box<dyn std::error::Error>> {
    let mut ponysay = Command::cargo_bin(env!("CARGO_PKG_NAME"))?;
    ponysay.arg("-l");
    let mut python_ponysay = python_ponysay();
    python_ponysay.arg("-l");

    assert_eq!(ponysay.output()?, python_ponysay.output()?);

    Ok(())
}

fn stty() -> Command {
    let mut cmd = Command::new("stty");
    cmd.arg("size");
    return cmd;
}

fn python_ponysay() -> Command {
    let mut python_ponysay = Command::new("python");
    python_ponysay.arg("python-ponysay/src/__main__.py");
    return python_ponysay;
}
