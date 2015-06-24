extern crate editline;
use editline::*;

const CMDS : [&'static str; 8] = [
    "foo ", "bar ", "bsd ", "cli ",
    "ls ", "cd ", "malloc ", "tee "
];

fn list_possib(line: &str) -> Vec<&str> {
    let mut matches = Vec::<&str>::new();

    for cmd in &CMDS {
        if cmd.starts_with(line) {
            matches.push(cmd)
        }
    }

    matches
}

fn complete(line: &str) -> Option<&str> {
    let possib = list_possib(line);

    match possib.len() {
        1 => Some(&possib[0][line.len()..]),
        _ => None,
    }
}

extern fn do_exit() -> Status {
    println!("Bye bye!");
    return Status::EOF;
}

fn main() {
    set_list_possib(list_possib);
    set_complete(complete);

    bind_key(Key::Ctrl('d'), do_exit);

    assert!(read_history("/tmp/editrs.history").is_ok());

    loop {
        match readline("cli> ") {
            Some(line) => println!("\t\t\t|{}|", line),
            None => break
        }
    }

    assert!(write_history("/tmp/editrs.history").is_ok());
}
