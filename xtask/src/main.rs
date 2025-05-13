use std::process::{Command, exit};

fn main() {
    let path = std::env::current_dir().unwrap();
    println!("[INFO] The current directory is {}", path.display());

    let status = Command::new("cargo")
        .args(&["build", "--release", "--manifest-path", "./Cargo.toml"])
        .current_dir("./dll")
        .status()
        .expect("Failed to build");
    if !status.success() {
        exit(1);
    }

    let status = Command::new("build-deps/pe_to_shellcode/pe2shc/Release/pe2shc.exe")
        .args(&["dll_rs.dll"])
        .current_dir("./target/release")
        .status()
        .expect("Failed to run pe2shc.exe");
    if !status.success() {
        exit(1);
    }

    println!("Done");
}
