use std::env;

// for reading hist file
use std::fs::File;
use std::io::prelude::*;

use termion::event::Key;
use termion::input::TermRead;
use termion::color;
use termion::raw::IntoRawMode;
use std::io::{Write, BufReader, stdout, stdin};


pub fn cd(dir_name: &String) -> std::io::Result<()> {
    let path = env::current_dir()?.join(dir_name);
    return env::set_current_dir(&path);
}


pub fn term_handler(hist: &Vec<String>, _path: &String) -> String {

    let (x, y) = term_cursor::get_pos().expect("");

    let stdin = stdin();

    let mut stdout = stdout().into_raw_mode().unwrap();
    
    // command buffer
    let mut buf = String::new();

    // temporary buffer for arrow history controls and history search
    let mut tbuf = String::new();

    // history position
    let mut hpos: usize = 0;

    let mut hsearch: bool = false;

    // buffer for history search
    let mut hsbuf = String::new();

    // index for history search
    let mut hsindex = hist.len();

    // index for buffer for left/right arrow key controls (goes backwards)
    let mut bufindex = 0;

    // tab search
    // let mut tsearch: bool = false;

    for c in stdin.keys() {
        let (cx, cy) = term_cursor::get_pos().expect("");
        match c.unwrap() {
            Key::Char(c) => {
                match c {
                    '\n' => {
                        write!(stdout, "\n{}{}",
                               term_cursor::Goto(1,y+1),
                               termion::clear::UntilNewline).unwrap();
                        break;
                    },
                    '\t' => {
                        // buf = tab_complete(&buf, &path, &mut stdout, &y);
                    },
                    _ => {
                        // write main buffer resp. history search buffer
                        if !hsearch {
                            let tmp = buf.len()-bufindex;
                            buf.insert(tmp, c);
                            write!(stdout, "{}{}{}", term_cursor::Goto(x,y), buf,
                                   term_cursor::Goto(cx+1,cy)).unwrap();
                        }
                        else {
                            hsbuf.push(c);
                            write!(stdout, "{}", c);
                        }
                    }
                };
            },
            // Key::Alt(c) => {},
            Key::Ctrl(c) => {
                match c {
                    'd' => return "exit".to_string(),
                    'l' => {
                        write!(stdout, "{}{}",
                               termion::clear::All,
                               term_cursor::Goto(1,1)).unwrap();
                        break;
                    },
                    'g' => {
                        // discard current input, enter new line
                        buf = "".to_string();
                        write!(stdout, "\n");
                        break;
                    }
                    'r' => {
                        if hsbuf == "".to_string() {
                            hsbuf.clone_from(&buf);
                        }
                        if hist.len() > 0 && hsbuf != "" {
                            for _ in 0..hsindex {
                                hsindex -= 1;
                                if hist[hsindex].contains(&hsbuf) {
                                    buf.clone_from(&hist[hsindex]);
                                    break;
                                }
                            }
                        }
                        write!(stdout, "{}$ {}{}{}{}\n{}bck-i-search: {}",
                               term_cursor::Goto(1,y),
                               color::Fg(color::Green),
                               buf,
                               color::Fg(color::Reset),
                               termion::clear::UntilNewline,
                               term_cursor::Goto(1,y+1),
                               hsbuf);
                        hsearch = true;
                    }
                    's' => {
                        if hsbuf != "" {
                            for _ in 1..hist.len()-hsindex {
                                hsindex += 1;
                                if hist[hsindex].contains(&hsbuf) {
                                    buf.clone_from(&hist[hsindex]);
                                    break;
                                }
                            }
                        }
                        write!(stdout, "{}$ {}{}{}{}\n{}fwd-i-search: {}",
                               term_cursor::Goto(1,y),
                               color::Fg(color::Green),
                               buf,
                               color::Fg(color::Reset),
                               termion::clear::UntilNewline,
                               term_cursor::Goto(1,y+1),
                               hsbuf);
                        hsearch = true;
                    }
                    _ => {}
                };
            },
            Key::Left => {
                if !hsearch && bufindex < buf.len() {
                    bufindex += 1;
                    write!(stdout, "{}", term_cursor::Left(1)); 
                }
            },
            Key::Right => {
                if !hsearch && bufindex > 0 {
                    bufindex -= 1;
                    write!(stdout, "{}", term_cursor::Right(1));
                }
            },
            Key::Up => {
                if hpos == 0 {
                        tbuf.clone_from(&buf);
                }
                if hpos < hist.len() {
                    hpos += 1;
                    buf.clone_from(&hist[hist.len()-hpos]);
                    write!(stdout, "{}{}{}",
                           term_cursor::Goto(x,y),
                           termion::clear::UntilNewline, buf);
                }
            },
            Key::Down => {
                if hpos > 0 {
                    hpos -= 1;
                }
                if hpos == 0 {
                    buf.clone_from(&tbuf);
                    write!(stdout, "{}$ {}{}",
                           term_cursor::Goto(1,y),
                           termion::clear::UntilNewline, buf);
                }
                else {
                    buf.clone_from(&hist[hist.len()-hpos]);
                    write!(stdout, "{}$ {}{}",
                           term_cursor::Goto(1,y),
                           termion::clear::UntilNewline, buf);
                }
            },
            Key::Backspace => {
                if hsearch {
                    hsbuf.pop();
                    tbuf = "".to_string();
                }
                else if bufindex < buf.len() {
                    let tmp: usize = buf.len()-bufindex-1;
                    buf.remove(tmp);
                }
                if cy == y && cx > 3 && !hsearch {
                    write!(stdout, "{}{}{}{}",
                           term_cursor::Goto(x,y),
                           termion::clear::UntilNewline,
                           buf,
                           term_cursor::Goto(cx-1,y));
                }
                else if cx > 15 {
                    write!(stdout, "{}{}",
                           term_cursor::Left(1),
                           termion::clear::UntilNewline);
                }
            },
            Key::Delete => {
                let (cx, cy) = term_cursor::get_pos().unwrap();
                if bufindex > 0 && cy == y {
                    let tmp: usize = buf.len()-bufindex;
                    
                    buf.remove(tmp);
                    bufindex -= 1;
                    
                    write!(stdout, "{}{}{}{}",
                           term_cursor::Goto(x,y),
                           termion::clear::UntilNewline,
                           buf,
                           term_cursor::Goto(cx,y));
                    
                }
            },
            _ => break
        }
        stdout.flush().unwrap();
    }

    return buf.trim_start().trim_end().to_string();
}


pub fn get_hist(hist: &mut Vec<String>) {

    // get history from previous sessions
    let f = File::open(".rhistory");
    match f {
        Err(_e) => {},
        _ => {
            let reader = BufReader::new(f.unwrap());
            for line in reader.lines() {
                hist.push(line.unwrap());
            }
        }
    }
}


// write history of current session to .rhistory
pub fn write_hist(path: std::path::PathBuf,hist: &mut Vec<String>) {
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .open(path.join(".rhistory"))
        .unwrap();
    for s in hist {
        write!(file, "{}\n", s);
    }
}


/*
fn tab_complete(buf: &String, path: &String, 
                stdout: &mut std::io::Stdout, y: &i32) -> String {

    let mut cbuf = buf.clone();

    for p in path.split(":") {
        
    }
    
    return cbuf;
}
*/ 
