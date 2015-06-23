extern crate editline;
use editline::*;

const CMDS : [&'static str; 8] = [
    "foo ", "bar ", "bsd ", "cli ",
    "ls ", "cd ", "malloc ", "tee "
];

extern fn do_exit() -> Status {
    println!("Bye bye!");
    return Status::EOF;
}

fn list_possib(line: &str) -> Vec<&str> {
    let mut matches = Vec::<&str>::new();

    for cmd in &CMDS {
        if cmd.starts_with(line) {
            matches.push(cmd)
        }
    }

    matches
}

fn main() {
    set_list_possib(list_possib);
    
    bind_key(Key::Meta('d'), do_exit);

    assert!(read_history("/tmp/editrs.history").is_ok());
    
    loop {
        let line = readline("test> ");

        println!("got: {}", line);
        if line == "exit" {
            break;
        }
    }

    assert!(write_history("/tmp/editrs.history").is_ok());
}
