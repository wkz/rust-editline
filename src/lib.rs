extern crate libc;

use libc::{c_char, c_int, c_void, free, malloc, size_t};

use std::ffi;
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
        pub static rl_line_buffer : *mut c_char;

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

fn cstr_to_str<'a>(c_s: *const c_char) -> Option<&'a str> {
    if c_s.is_null() {
        return None
    }

    let _s = unsafe { ffi::CStr::from_ptr(c_s) };

    str::from_utf8(_s.to_bytes()).ok()
}

fn str_to_dup_cstr(s: &str) -> Option<*mut c_char> {
    let c_s = match ffi::CString::new(s) {
        Ok(c_s) => c_s,
        Err(..) => return None
    };

    let dup = unsafe { malloc((s.len() + 1) as size_t) as *mut c_char };
    if dup.is_null() {
        return None;
    }

    Some(unsafe { libc::strcpy(dup, c_s.as_ptr()) })
}


pub fn line_buffer<'a>() -> Option<&'a str> {
    cstr_to_str(c::rl_line_buffer)
}

pub fn readline(prompt: &str) -> Option<&str> {
    let c_prompt = match ffi::CString::new(prompt) {
        Ok(c_prompt) => c_prompt,
        Err(..) => return None
    };

    let c_line = unsafe { c::readline(c_prompt.as_ptr()) };

    cstr_to_str(c_line)
}


pub fn read_history(filename: &str) -> bool {
    let c_filename = match ffi::CString::new(filename) {
        Ok(c_filename) => c_filename,
        Err(..) => return false
    };

    match unsafe { c::read_history(c_filename.as_ptr()) } {
        0 => true,
        _ => false
    }
}

pub fn write_history(filename: &str) -> bool {
    let c_filename = match ffi::CString::new(filename) {
        Ok(c_filename) => c_filename,
        Err(..) => return false
    };

    match unsafe { c::write_history(c_filename.as_ptr()) } {
        0 => true,
        _ => false
    }
}

pub fn add_history(line: &str) -> bool {
    let c_line = match ffi::CString::new(line) {
        Ok(c_line) => c_line,
        Err(..) => return false
    };

    unsafe { c::add_history(c_line.as_ptr()) };
    true
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


pub type ListPossibFn = fn(word: &str) -> Vec<&str>;
static mut list_possib_fn : Option<ListPossibFn> = None;

extern fn list_possib_bridge(c_word: *const c_char, c_possib_ptr: *mut*mut*mut c_char) -> c_int {
    use std::slice;

    let word = match cstr_to_str(c_word) {
        Some(word) => word,
        None => return 0 as c_int
    };

    let possib_fn = unsafe { list_possib_fn.unwrap() };
    let possib = possib_fn(word);

    let c_possib_sz = (mem::size_of::<*mut c_char>() * possib.len()) as size_t;
    let c_possib = unsafe {
        let mem = malloc(c_possib_sz) as *mut*mut c_char;
        if mem.is_null() {
            return 0 as c_int;
        }

        slice::from_raw_parts_mut(mem, possib.len())
    };

    let mut ok = 0;

    for i in 0..possib.len() {
        match str_to_dup_cstr(possib[i]) {
            Some(c_entry) => {
                c_possib[ok] = c_entry;
                ok += 1;
            },

            None => ()
        }
    }

    if ok == 0 {
        unsafe { free(c_possib.as_ptr() as *mut c_void) };
        return 0 as c_int;
    }

    unsafe {
        *c_possib_ptr = c_possib.as_ptr() as *mut*mut c_char;
    }
    return ok as c_int;
}

pub fn set_list_possib(cb: ListPossibFn) {
    unsafe {
        list_possib_fn = Some(cb);
        c::rl_set_list_possib_func(list_possib_bridge);
    }
}


pub type CompleteFn = fn(word: &str) -> Option<&str>;
static mut complete_fn : Option<CompleteFn> = None;

extern fn complete_bridge(c_word: *const c_char, found: *mut c_int) -> *mut c_char {
    let word = match cstr_to_str(c_word) {
        Some(word) => word,
        None => return ptr::null_mut::<c_char>()
    };

    let complete = unsafe { complete_fn.unwrap() };

    let text = match complete(word) {
        Some(text) => text,
        None => return ptr::null_mut::<c_char>()
    };

    match str_to_dup_cstr(text) {
        Some(c_text) => { unsafe { *found = 1 }; c_text },
        None => ptr::null_mut::<c_char>()
    }
}

pub fn set_complete(cb: CompleteFn) {
    unsafe {
        complete_fn = Some(cb);
        c::rl_set_complete_func(complete_bridge);
    }
}
