extern crate editline;
use editline::*;

const CMDS: [&'static str; 8] = [
    "foo ", "bar ", "bsd ", "cli ", "ls ", "cd ", "malloc ", "tee ",
];

fn list_possib(word: &str) -> Vec<&str> {
    let mut matches = Vec::<&str>::new();

    for cmd in &CMDS {
        if cmd.starts_with(word) {
            matches.push(cmd)
        }
    }

    matches
}

fn complete(word: &str) -> Option<&str> {
    let possib = list_possib(word);

    match possib.len() {
        1 => Some(&possib[0][word.len()..]),
        _ => None,
    }
}

extern "C" fn do_exit() -> Status {
    println!("Bye bye!");
    return Status::EOF;
}

fn main() {
    set_list_possible(list_possib);
    set_complete(complete);

    bind_key(Key::Ctrl('d'), do_exit);

    assert!(read_history("/tmp/editrs.history"));

    loop {
        match readline("cli> ") {
            Some(line) => println!("\t\t\t|{}|", line),
            None => break,
        }
    }

    assert!(write_history("/tmp/editrs.history"));
}
