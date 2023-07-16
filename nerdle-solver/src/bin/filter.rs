use std::fs::File;
use std::io::{BufRead, BufReader};

use nerdle_solver::mask;

fn prompt(s: &'static str) -> String {
    println!("{}", s);
    let mut line = String::new();
    std::io::stdin().read_line(&mut line).unwrap();
    line.trim().to_string()
}

fn main() {
    let args = std::env::args().collect::<Vec<_>>();
    println!("Reading options from {}", args[1]);
    let f = BufReader::new(File::open(&args[1]).unwrap());
    let mut options: Vec<String> = f.lines().map(|s| s.unwrap().trim().to_string()).collect();

    println!("Loaded {} options", options.len());

    loop {
        println!();
        let guess = prompt("Enter your guess (you can use s for ² and c for ³)");
        let mask_txt =
            prompt("Enter your mask (G or 2 for green; P or 1 for purple; B or 0 for black)");
        if let Some(m) = mask::parse_mask_results(&guess, &mask_txt) {
            options.retain(|o| mask::matches_mask(o, &m));
            println!("{} options remaining", options.len());
            println!();
            for v in options.iter().take(25) {
                println!("- {}", v);
            }
            if options.len() > 25 {
                println!("- ...");
            }

            if options.len() == 1 {
                return;
            }
        }
    }
}
