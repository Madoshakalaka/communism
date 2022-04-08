use std::process::{Command, Stdio};

fn main() {
    Command::new("cross")
        .arg("build")
        .arg("--release")
        .arg("-p")
        .arg("sentinel")
        .arg("--target")
        .arg("aarch64-unknown-linux-gnu")
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .output()
        .unwrap();

    Command::new("ssh")
        .arg("root@xray")
        .arg("systemctl")
        .arg("stop")
        .arg("sentinel")
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .output()
        .unwrap();

    Command::new("scp")
        .arg("target/aarch64-unknown-linux-gnu/release/sentinel")
        .arg("root@xray:/opt/sentinel/sentinel")
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .output()
        .unwrap();

    Command::new("ssh")
        .arg("root@xray")
        .arg("systemctl")
        .arg("restart")
        .arg("sentinel")
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .output()
        .unwrap();
}
