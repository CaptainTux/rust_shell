extern crate termion;
extern crate term_cursor;

mod shell;

use std::env;

use std::io::{Write, stdout};

use std::process::Command;

fn main() {

    let mut stdout = stdout();  // not needed, as one can use println!
    // let mut stdout_lock = stdout.lock();

    // let prompt = "%>";

    let mut hist: Vec<String> = Vec::new();

    stdout.write(b"\x1b[H\x1b[2J").expect("");

    stdout.write(b"Rust shell version 1.0\n").expect("");

    let mut path = String::new();

    for (key, value) in env::vars_os() {
        if key == "PATH" {
            path = value.into_string().unwrap();
        }
    }

    let curr_path = env::current_dir().unwrap();

    shell::get_hist(&mut hist);

    loop {

        // let path = env::current_dir().unwrap();
        // println!("{:?} > ", path.display());

        let (_x, y) = term_cursor::get_pos().expect("");

        stdout.flush().unwrap();

        write!(stdout, "{}$ ", term_cursor::Goto(1, y));  // finally a working prompt

        stdout.flush().unwrap();

        let input = shell::term_handler(&mut hist, &path);

        stdout.flush().unwrap();        
        
        // stdin.lock().read_line(&mut input).expect("");  // read input

        if hist.last() != Some(&input) {
            if input.trim() == "exit".to_string() {
                shell::write_hist(curr_path, &mut hist);
                break;
            }
            /* for i in 0..hist.len() {
                print!("{}", hist[i]);
            } */
            hist.push(input.clone());
        }

        stdout.flush().unwrap();

        let mut iter = input.split_whitespace();  // split program name from arguments

        if let Some(prog) = iter.next() {  // get program name, if any
            if prog == "cd" {
                if let Some(dir) = iter.next() {  // a little bit dirty, since we disregard all arguments after the first...
                    if let Err(_e) = shell::cd(&dir.to_string()) {  // catch error, if any
                        println!("cd: no such file or directory: {}", dir);  // print error
                    }
                }
            }
            else {
                let status = Command::new(prog)
                    .args(iter.collect::<Vec<_>>())
                    .status();
                if let Err(_e) = status {  // catch error, if any
                    hist.pop();
                    println!("rsh: command not found: {}", prog);  // cannot use .expect("*"), because cannot insert program name into "*"
                }
            }
        }
    }
}
