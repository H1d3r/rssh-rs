#![allow(unused_imports)]

use std::ptr::null_mut;
use dll_rs::{dll_main, dll_start, DllMain};
use tokio::runtime::Runtime;
/// Executable wrapper for functionality in dll_rs
#[tokio::main]
async fn main() {
    // Initialize resources, etc.
    unsafe { dll_start() };
}


