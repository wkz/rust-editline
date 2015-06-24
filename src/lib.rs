extern crate libc;

use libc::{c_char, c_int, malloc, size_t};

use std::ffi;
use std::io;
use std::mem;
use std::ptr;
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

    type CompleteFn = extern fn(*const c_char, *mut c_int) -> *mut c_char;
    type ListPossibFn = extern fn(*const c_char, *mut*mut*mut c_char) -> c_int;

    #[link(name = "editline")]
    extern {
        pub fn readline(prompt: *const c_char) -> *mut c_char;

        pub fn read_history(filename: *const c_char) -> c_int;
        pub fn write_history(filename: *const c_char) -> c_int;
        pub fn add_history(line: *const c_char);

        pub fn el_bind_key(key: c_int, cb: extern fn()->Status);
        pub fn el_bind_key_in_metamap(key: c_int, function: extern fn()->Status);

        pub fn rl_set_complete_func(cb: CompleteFn) -> CompleteFn;
        pub fn rl_set_list_possib_func(cb: ListPossibFn) -> ListPossibFn;
    }
}


pub fn readline(prompt: &str) -> Option<&str> {
    let c_prompt = ffi::CString::new(prompt);
    if c_prompt.is_err() {
        return None
    }

    let c_line_ptr = unsafe { c::readline(c_prompt.unwrap().as_ptr()) };
    if c_line_ptr.is_null() {
        return None
    }

    let c_line = unsafe { ffi::CStr::from_ptr(c_line_ptr) };

    match str::from_utf8(c_line.to_bytes()) {
        Ok(line) => Some(line),
        Err(..) => None
    }
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
    } as i32;

    match key {
        Key::Plain(..) | Key::Ctrl(..) => unsafe {
            c::el_bind_key(byte, callback);
        },
        Key::Meta(..) | Key::MetaCtrl(..) => unsafe {
            c::el_bind_key_in_metamap(byte, callback);
        },
    }
}


pub type ListPossibFn = fn(line: &str) -> Vec<&str>;
static mut list_possib_fn : Option<ListPossibFn> = None;

extern fn list_possib_bridge(c_line: *const c_char, c_possib_ptr: *mut*mut*mut c_char) -> c_int {
    use std::slice;

    let line_cstr = unsafe { ffi::CStr::from_ptr(c_line) };
    let line = str::from_utf8(line_cstr.to_bytes()).unwrap();

    let possib_fn = unsafe { list_possib_fn.unwrap() };
    let possib = possib_fn(line);

    let c_possib_sz = (mem::size_of::<*mut c_char>() * possib.len()) as size_t;
    let c_possib = unsafe {
        slice::from_raw_parts_mut(malloc(c_possib_sz) as *mut*mut c_char, possib.len())
    };

    for i in 0..possib.len() {
        let entry_cstr = ffi::CString::new(possib[i]).unwrap();

        c_possib[i] = unsafe { malloc((possib[i].len() + 1) as size_t) as *mut c_char };

        unsafe { libc::strcpy(c_possib[i], entry_cstr.as_ptr()) };
    }

    unsafe {
        *c_possib_ptr = c_possib.as_ptr() as *mut*mut c_char;
    }
    return possib.len() as c_int;
}

pub fn set_list_possib(cb: ListPossibFn) {
    unsafe {
        list_possib_fn = Some(cb);
        c::rl_set_list_possib_func(list_possib_bridge);
    }
}


pub type CompleteFn = fn(line: &str) -> Option<&str>;
static mut complete_fn : Option<CompleteFn> = None;

extern fn complete_bridge(c_line: *const c_char, found: *mut c_int) -> *mut c_char {
    let line_cstr = unsafe { ffi::CStr::from_ptr(c_line) };
    let line = str::from_utf8(line_cstr.to_bytes()).unwrap();

    let complete = unsafe {
        match complete_fn {
            None => None,
            Some(cb) => cb(line)
        }
    };

    match complete {
        None => ptr::null_mut::<c_char>(),
        Some(text) => {
            let cstr_text = ffi::CString::new(text).unwrap();

            unsafe {
                let c_text = malloc((text.len() + 1) as size_t) as *mut c_char;

                *found = 1;
                libc::strcpy(c_text, cstr_text.as_ptr())
            }
        }
    }
}

pub fn set_complete(cb: CompleteFn) {
    unsafe {
        complete_fn = Some(cb);
        c::rl_set_complete_func(complete_bridge);
    }
}
