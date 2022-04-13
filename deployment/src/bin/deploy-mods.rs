use std::process::{Command, Stdio};

fn main() {
    Command::new("zip")
        .arg("-r") // recursive
        .arg("commupack.zip")
        .arg("commupack/")
        .stdout(Stdio::inherit())
        .output()
        .unwrap();

    Command::new("scp")
        .arg("commupack.zip")
        .arg("xray:/site/commupack.zip")
        .stdout(Stdio::inherit())
        .output()
        .unwrap();
}
