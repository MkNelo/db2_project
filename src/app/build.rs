const ROOT: &'static str = "./*";
const CLIENT_ROOT: &'static str = "../../client/src";

use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed={}", ROOT);
    println!("cargo:rerun-if-changed={}", CLIENT_ROOT);
    
    let mut command = Command::new("elm");
    command
    .args(&["make", &format!("{}/Main.elm", CLIENT_ROOT), "--output=elm.js"])
    .output()
    .expect("Build failed.");
}