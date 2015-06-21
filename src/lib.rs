extern crate libc;

use std::ffi;
use std::io;
use std::str;

mod c {
    use libc::{c_char, c_int};

    #[repr(C)]
    pub enum KeyResult {
        Done = 0,
        EOF,
        Move,
        Dispatch,
        Stay,
        Signal
    }

    #[link(name = "editline")]
    extern {
        pub fn readline(prompt: *const c_char) -> *mut c_char;

        pub fn read_history(filename: *const c_char) -> c_int;
        pub fn write_history(filename: *const c_char) -> c_int;
        pub fn add_history(line: *const c_char);

        pub fn el_bind_key(key: c_int, function: extern fn()->KeyResult);
        pub fn el_bind_key_in_metamap(key: c_int, function: extern fn()->KeyResult);
    }
}

pub fn readline(prompt: &str) -> &str {
    let cprompt = ffi::CString::new(prompt).unwrap();

    let line = unsafe {
        ffi::CStr::from_ptr(c::readline(cprompt.as_ptr()))
    };

    str::from_utf8(line.to_bytes()).unwrap()
}

pub fn read_history(filename: &str) -> Result<(), io::Error> {
    let cfilename = ffi::CString::new(filename).unwrap();

    let errno = unsafe {
        c::read_history(cfilename.as_ptr())
    };

    if errno == 0 {
        return Ok(());
    } else {
        return Err(io::Error::new(io::ErrorKind::Other,
                                  format!("Unable read history file from {}", filename)));
    }
}

pub fn write_history(filename: &str) -> Result<(), io::Error> {
    let cfilename = ffi::CString::new(filename).unwrap();

    let errno = unsafe {
        c::write_history(cfilename.as_ptr())
    };

    if errno == 0 {
        return Ok(());
    } else {
        return Err(io::Error::new(io::ErrorKind::Other,
                                  format!("Unable write history file to {}", filename)));
    }
}

pub fn add_history(line: &str) {
    let cline = ffi::CString::new(line).unwrap();

    unsafe {
        c::add_history(cline.as_ptr())
    };
}

pub enum Mod {
    None,
    Control,
    Meta,
    MetaControl,
}

pub use self::c::KeyResult;

pub fn bind_key(modifier: Mod, key: char, callback: extern fn()->KeyResult) {
    let mut byte = key as u8;
    
    match modifier {
        Mod::Control | Mod::MetaControl => byte &= 0x1f,
        _ => {},
    }

    match modifier {
        Mod::None | Mod::Control => unsafe {
            c::el_bind_key(byte as i32, callback);
        },
        Mod::Meta | Mod::MetaControl => unsafe {
            c::el_bind_key_in_metamap(byte as i32, callback);
        },
    }
}
