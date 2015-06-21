extern crate editline;

extern fn do_exit() -> editline::KeyResult {
    println!("should exit now");
    return editline::KeyResult::EOF;
}

fn main() {
    editline::bind_key(editline::Mod::Meta, 'd', do_exit);

    assert!(editline::read_history("/tmp/editrs.history").is_ok());
    
    loop {
        let line = editline::readline("test> ");

        println!("got: {}", line);
        if line == "exit" {
            break;
        }
    }

    assert!(editline::write_history("/tmp/editrs.history").is_ok());
}
