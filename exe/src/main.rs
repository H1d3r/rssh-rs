#![allow(unused_imports)]
use dll_rs::dll_main;

/// Executable wrapper for functionality in dll_rs
fn main() {
    #[cfg(not(debug_assertions))]
    println!("[!] RELEASE DEBUGGING EXECUTABLE [!]");

    dll_main();
}
