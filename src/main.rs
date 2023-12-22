use std::io::{self, Read};
use std::os::windows::io::{
FromRawHandle};
use std::ptr;
use winapi::um::handleapi::{CloseHandle};
use winapi::um::processthreadsapi::{CreateProcessW, PROCESS_INFORMATION, STARTUPINFOW};
use winapi::um::winbase::{CREATE_NO_WINDOW, STARTF_USESTDHANDLES};
use winapi::um::winnt::{HANDLE};
use winapi::um::namedpipeapi::CreatePipe;
use winapi::um::securitybaseapi::InitializeSecurityDescriptor;

use winapi::um::winnt::{
    SECURITY_DESCRIPTOR, 
    SECURITY_DESCRIPTOR_REVISION
};
use winapi::um::minwinbase::SECURITY_ATTRIBUTES;


fn execute_command(command: &str) -> io::Result<Vec<u8>> {
    let mut h_read_pipe: HANDLE = ptr::null_mut();
    let mut h_write_pipe: HANDLE = ptr::null_mut();
    let mut sa_attr: SECURITY_ATTRIBUTES = unsafe { std::mem::zeroed() };
    let mut pi_proc_info: PROCESS_INFORMATION = unsafe { std::mem::zeroed() };
    let mut si_start_info: STARTUPINFOW = unsafe { std::mem::zeroed() };

    // Initialize the SECURITY_ATTRIBUTES
    let mut sa_attr: SECURITY_ATTRIBUTES = unsafe { std::mem::zeroed() };
    let mut sd: SECURITY_DESCRIPTOR = unsafe { std::mem::zeroed() };
    unsafe {
        InitializeSecurityDescriptor(&mut sd as *mut _ as *mut _, SECURITY_DESCRIPTOR_REVISION);
        sa_attr.nLength = std::mem::size_of::<SECURITY_ATTRIBUTES>() as u32;
        sa_attr.lpSecurityDescriptor = &mut sd as *mut _ as *mut _;
        sa_attr.bInheritHandle = 1; // TRUE
    }

    
    // Create an anonymous pipe
    unsafe {
        CreatePipe(&mut h_read_pipe, &mut h_write_pipe, &mut sa_attr, 0);
    }

    // Set the startup information so that the new process inherits the pipe handle
    si_start_info.cb = std::mem::size_of::<STARTUPINFOW>() as u32;
    si_start_info.hStdOutput = h_write_pipe;
    si_start_info.hStdError = h_write_pipe;
    si_start_info.dwFlags = STARTF_USESTDHANDLES;

    // Convert command to wide string for CreateProcessW
    let mut wide_cmd: Vec<u16> = command.encode_utf16().collect();
    wide_cmd.push(0);

    unsafe {
        if CreateProcessW(
            ptr::null_mut(),
            wide_cmd.as_mut_ptr(),
            ptr::null_mut(),
            ptr::null_mut(),
            1, // bInheritHandles
            CREATE_NO_WINDOW,
            ptr::null_mut(),
            ptr::null_mut(),
            &mut si_start_info,
            &mut pi_proc_info,
        ) == 0 {
            CloseHandle(h_read_pipe);
            CloseHandle(h_write_pipe);
            return Err(io::Error::last_os_error());
        }
    }

    unsafe { CloseHandle(h_write_pipe) };

    // Read from the pipe
    let mut result_buffer = Vec::new();
    let mut pipe_reader = unsafe { std::fs::File::from_raw_handle(h_read_pipe as *mut _) };

    pipe_reader.read_to_end(&mut result_buffer)?;

    unsafe {
        CloseHandle(h_read_pipe);
        CloseHandle(pi_proc_info.hProcess);
        CloseHandle(pi_proc_info.hThread);
    }

    Ok(result_buffer)
}

fn main() {
    use std::env::Args;
    let args: Vec<String> = std::env::args().collect() ;
    let command = args.get(1).unwrap();
    println!("execute command => \n {:?}" , command );
    match execute_command(command) {
        Ok(output) => println!("Command output => \n {:?}", String::from_utf8_lossy(&output).trim_end_matches(|c| c == '\r' || c == '\n').to_string()),
        Err(e) => eprintln!("Failed to execute command: {}", e),
    }
}
