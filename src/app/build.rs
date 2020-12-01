const ROOT: &'static str = "./src";
const PUBLIC: &'static str = "./public";
const DIST: &'static str = "./dist";
const CLIENT_ROOT: &'static str = "../../client/app/src";

use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed={}", ROOT);
    println!(
        "cargo:rerun-if-changed={}/{build}",
        ROOT,
        build = "build.rs"
    );
    println!("cargo:rerun-if-changed={}", PUBLIC);
    println!("cargo:rerun-if-changed={}", CLIENT_ROOT);

    let mut command = Command::new("elm");
    let output = command
        .args(&[
            "make",
            &format!("{}/Main.elm", CLIENT_ROOT),
            &format!("--output={}/elm.js", DIST),
        ])
        .output()
        .expect("Build failed.");

    if !output.status.success() {
        panic!(
            "Build failed with code: {code}, with output: {output}",
            code = output
                .status
                .code()
                .expect("Build failed because terminated"),
            output = String::from_utf8(output.stderr).unwrap()
        )
    }
}
