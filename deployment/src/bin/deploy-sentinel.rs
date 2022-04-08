use std::process::{Command, Stdio};

fn main() {
    Command::new("cargo")
        .arg("build")
        .arg("--release")
        .arg("-p")
        .arg("sentinel")
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .output()
        .unwrap();

    Command::new("scp")
        .arg("target/release/sentinel")
        .arg("root@xray:/opt/sentinel/sentinel")
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .output()
        .unwrap();
}
