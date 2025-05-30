use std::process::{Command, exit};

fn main() {
    let path = std::env::current_dir().unwrap();
    println!("[XTASK] The current directory is {}", path.display());

    // Building Dll
    println!("[XTASK] Building dll...");
    let status = Command::new("cargo")
        .args(&["build", "--release", "--manifest-path", "./Cargo.toml"])
        .current_dir("./dll")
        .status()
        .expect("Failed to build");
    if !status.success() {
        exit(1);
    }

    // Apply reflective loader
    println!("[XTASK] Applying pe2shc loader...");
    let status = Command::new("pe2shc.exe")
        .args(&["dll_rs.dll"])
        .current_dir("target/x86_64-pc-windows-msvc/release")
        .status()
        .expect("Failed to run pe2shc.exe");
    if !status.success() {
        exit(1);
    }

    // Copy reflective dll
    println!("[XTASK] Copying reflective dll...");
    let status = Command::new("cp")
        .args(&["target/x86_64-pc-windows-msvc/release/dll_rs.shc.dll", "bins/x64/dll_rs.shc.dll"])
        .current_dir(".")
        .status()
        .expect("Failed to run pe2shc.exe");
    if !status.success() {
        exit(1);
    }

    // Compile BOF
    println!("[XTASK] Building BOF...");
    let status = Command::new("cc")
        .args(&["bof.c", "-c", "-o", "../bins/x64/bof_write_pipe.x64.o"])
        .current_dir("./bof-write-pipe")
        .status()
        .expect("Failed to build");
    if !status.success() {
        exit(1);
    }
    
    println!("Done");
}
