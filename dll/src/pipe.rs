#![allow(unused_imports)]
use crate::windows::*;
use std::ffi::CString;
use std::ptr;
use std::ptr::null_mut;

pub const MAX_PIPE_BUFFER_SIZE: usize = 4096;
pub static OUTPUT_PIPE_NAME: &[u8; 42] = b"\\\\.\\pipe\\OUTPUT_PIPE_NAME_NO_CHANGE_PLS\0\0\0";
pub static INPUT_PIPE_NAME: &[u8; 42] = b"\\\\.\\pipe\\INPUT_PIPE_NAME_NO_CHANGE_PLS\0\0\0\0";

pub(crate) fn read_input(h_input_pipe: HANDLE) -> Option<String> {
    let mut dyn_buffer = Box::new(vec![0u8; MAX_PIPE_BUFFER_SIZE as usize]);

    let mut bytes_read: u32 = 0;
    let ptr_bytes_read: *mut u32 = ptr::addr_of_mut!(bytes_read);

    let mut total_bytes_avail: u32 = 0;
    let ptr_total_bytes_avail: *mut u32 = ptr::addr_of_mut!(total_bytes_avail);

    let mut bytes_left: u32 = 0;
    let ptr_bytes_left: *mut u32 = ptr::addr_of_mut!(bytes_left);

    let mut counter = 0;

    loop {
        let peek_result = unsafe {
            PeekNamedPipe(
                h_input_pipe,
                dyn_buffer.as_ptr() as *mut u8,
                MAX_PIPE_BUFFER_SIZE as u32,
                ptr_bytes_read,
                ptr_total_bytes_avail,
                ptr_bytes_left
            )
        };

        if peek_result > 0 && bytes_left > 0 {
            // Check if our buffer is big enough
            if bytes_left > MAX_PIPE_BUFFER_SIZE as u32 {
                dyn_buffer.resize(bytes_left as usize, 0);
            }

            // Read the data
            let read_result = unsafe {
                ReadFile(
                    h_input_pipe,
                    dyn_buffer.as_ptr() as *mut u8,
                    bytes_left,
                    ptr_bytes_read,
                    null_mut()
                )
            };

            // If read was successful and we got all expected bytes
            if read_result > 0 && bytes_left == bytes_read {
                // Convert only the read portion of the buffer to string
                return String::from_utf8(dyn_buffer[..bytes_read as usize].to_vec())
                    .ok()
                    .map(|s| s.to_string());
            }
        } else if peek_result == 0 {
            counter += 1;
            std::thread::sleep(std::time::Duration::from_millis(100));
            dbg!("Waiting for data...");

            if counter > 100 {
                break;
            }
        }
    }

    None // Return None if we timeout or encounter an error
}

pub(crate) fn initialize_input_pipe() -> Option<HANDLE> {
    let pipe_name = String::from_utf8_lossy(&*INPUT_PIPE_NAME);

    let h_pipe = unsafe {
        CreateNamedPipeA(
            pipe_name.as_ptr() as *const u8,
            PIPE_ACCESS_DUPLEX,
            PIPE_TYPE_BYTE,
            1,
            MAX_PIPE_BUFFER_SIZE as u32,
            MAX_PIPE_BUFFER_SIZE as u32,
            0,
            std::ptr::null_mut(),
        )
    };

    if h_pipe == INVALID_HANDLE_VALUE {
        let err = unsafe { GetLastError() };
        dbg!("CreateNamedPipe failed: {}", err);
        return None;
    }

    return Some(h_pipe);
}

pub(crate) fn initialize_output_pipe() -> Option<HANDLE> {
    let pipe_name = String::from_utf8_lossy(&*OUTPUT_PIPE_NAME);

    let h_pipe = unsafe {
        CreateNamedPipeA(
            pipe_name.as_ptr() as *const u8,
            PIPE_ACCESS_DUPLEX,
            PIPE_TYPE_BYTE,
            1,
            MAX_PIPE_BUFFER_SIZE as u32,
            MAX_PIPE_BUFFER_SIZE as u32,
            0,
            std::ptr::null_mut(),
        )
    };

    if h_pipe == INVALID_HANDLE_VALUE {
        let err = unsafe { GetLastError() };
        dbg!("CreateNamedPipe failed: {}", err);
        return None;
    }

    Some(h_pipe)
}

#[cfg(not(debug_assertions))]
pub(crate) fn write_output(h_output_pipe: HANDLE, data: &str) {
    let message = data.as_bytes();
    let mut bytes_written: u32 = 0;

    let connected = unsafe { ConnectNamedPipe(h_output_pipe, std::ptr::null_mut()) };
    if connected == 0 {
        let err = unsafe { GetLastError() };
        if err != ERROR_PIPE_CONNECTED {
            dbg!("ConnectNamedPipe failed: {}", err);
            unsafe { CloseHandle(h_output_pipe) };
            return;
        }
    }

    dbg!("[+] Beacon connected! Sending message...");

    let success = unsafe {
        WriteFile(
            h_output_pipe,
            message.as_ptr(),
            message.len() as u32,
            &mut bytes_written,
            std::ptr::null_mut(),
        )
    };

    if success == 0 {
        let err = unsafe { GetLastError() };
        dbg!("WriteFile failed: {}", err);
        unsafe { CloseHandle(h_output_pipe) };
        return;
    }

    unsafe {
        FlushFileBuffers(h_output_pipe);
    }
}

/// Debug implementation of write_output.
#[cfg(debug_assertions)]
pub(crate) fn write_output(_h_output_pipe: HANDLE, data: &str) {
    println!("{}", data);
}
