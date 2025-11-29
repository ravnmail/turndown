use std::io::{self, Read};
use turndown::Turndown;

fn main() {
    // Read HTML from stdin
    let mut html = String::new();
    match io::stdin().read_to_string(&mut html) {
        Ok(_) => {
            let turndown = Turndown::new();
            let markdown = turndown.convert(&html);

            println!("{}", markdown);
        }
        Err(e) => {
            eprintln!("Error reading from stdin: {}", e);
            std::process::exit(1);
        }
    }
}
