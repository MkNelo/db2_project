use std::process::*;

const TARGET_PATH: &'static str = "client/src";

fn format_to_target<S: AsRef<str>>(path: S) -> String {
    format!("{}/{}", TARGET_PATH, path.as_ref())
}

fn main() {
    println!("cargo:rerun-if-changed=src/*");
    println!("cargo:rerun-if-changed={}/*", TARGET_PATH);

    let mut client_build_command = Command::new("elm");

    let mut after_dir = || {
        client_build_command
            .args(&["make", &format_to_target("Main.elm")])
            .output()
    };

    after_dir()
        .and_then(|_| {
            Command::new("mv")
                .args(&["index.html", "./src/index.html"])
                .output()
        })
        .expect("Build failed");
}
/*
    change_directory
        .arg(TARGET_PATH)
        .output()
        .and_then(|output|
            panic!("Output: {}", String::from_utf8(output.stdout).unwrap())
        )
        .and_then(after_dir)
        .expect("Build process failed");
*/
