extern crate libc;
use libc::{c_char, c_int};

use std::ffi;
use std::io;
use std::str;

mod c {
    use libc::{c_char, c_int};

    #[repr(C)]
    pub enum Status {
        Done = 0,
        EOF,
        Move,
        Dispatch,
        Stay,
        Signal
    }

    // type CompleteFn = extern fn(*const c_char, *mut c_int) -> *mut c_char;
    type ListPossibFn = extern fn(*const c_char, *mut*mut*mut c_char) -> c_int;
    
    #[link(name = "editline")]
    extern {
        pub fn readline(prompt: *const c_char) -> *mut c_char;

        pub fn read_history(filename: *const c_char) -> c_int;
        pub fn write_history(filename: *const c_char) -> c_int;
        pub fn add_history(line: *const c_char);

        pub fn el_bind_key(key: c_int, cb: extern fn()->Status);
        pub fn el_bind_key_in_metamap(key: c_int, function: extern fn()->Status);

        // pub fn rl_set_complete_func(cb: CompleteFn) -> CompleteFn;
        pub fn rl_set_list_possib_func(cb: ListPossibFn) -> ListPossibFn;
    }
}

pub fn readline(prompt: &str) -> &str {
    let c_prompt = ffi::CString::new(prompt).unwrap();

    let line = unsafe {
        ffi::CStr::from_ptr(c::readline(c_prompt.as_ptr()))
    };

    str::from_utf8(line.to_bytes()).unwrap()
}

pub fn read_history(filename: &str) -> Result<(), io::Error> {
    let c_filename = ffi::CString::new(filename).unwrap();

    let errno = unsafe {
        c::read_history(c_filename.as_ptr())
    };

    if errno == 0 {
        return Ok(());
    } else {
        return Err(io::Error::new(io::ErrorKind::Other,
                                  format!("Unable read history file from {}", filename)));
    }
}

pub fn write_history(filename: &str) -> Result<(), io::Error> {
    let c_filename = ffi::CString::new(filename).unwrap();

    let errno = unsafe {
        c::write_history(c_filename.as_ptr())
    };

    if errno == 0 {
        return Ok(());
    } else {
        return Err(io::Error::new(io::ErrorKind::Other,
                                  format!("Unable write history file to {}", filename)));
    }
}

pub fn add_history(line: &str) {
    let c_line = ffi::CString::new(line).unwrap();

    unsafe {
        c::add_history(c_line.as_ptr())
    };
}

pub enum Key {
    Plain(char),
    Ctrl(char),
    Meta(char),
    MetaCtrl(char),
}

pub use self::c::Status;

pub fn bind_key(key: Key, callback: extern fn()->Status) {
    let byte = match key {
        Key::Ctrl(k) | Key::MetaCtrl(k) => (k as u8) & 0x1f,
        Key::Plain(k) | Key::Meta(k) => k as u8,
    };

    match key {
        Key::Plain(..) | Key::Ctrl(..) => unsafe {
            c::el_bind_key(byte as i32, callback);
        },
        Key::Meta(..) | Key::MetaCtrl(..) => unsafe {
            c::el_bind_key_in_metamap(byte as i32, callback);
        },
    }        
}

pub type ListPossibFn = fn(line: &str) -> Vec<&str>;
static mut list_possib_fn : Option<ListPossibFn> = None;

extern fn list_possib_bridge(c_line: *const c_char, c_possib: *mut*mut*mut c_char) -> c_int {
    let line_cstr = unsafe { ffi::CStr::from_ptr(c_line) };
    let line = str::from_utf8(line_cstr.to_bytes()).unwrap();

    let possib_fn = unsafe { list_possib_fn.unwrap() };
    let possib = possib_fn(line);

    // convert possib to char***
    
    return possib.len() as c_int;
}

pub fn set_list_possib(cb: ListPossibFn) {
    unsafe {
        list_possib_fn = Some(cb);
        c::rl_set_list_possib_func(list_possib_bridge);
    }
}
