use std::fs::File;
use std::io::{BufRead, BufReader};

use rand::{seq::SliceRandom, thread_rng};

use nerdle_solver::mask;

fn prompt(s: &'static str) -> String {
    println!("{}", s);
    let mut line = String::new();
    std::io::stdin().read_line(&mut line).unwrap();
    line.trim().to_string()
}

fn main() {
    let args = std::env::args().collect::<Vec<_>>();
    print!("Reading options from {}...", args[1]);
    let f = BufReader::new(File::open(&args[1]).unwrap());
    let mut options: Vec<String> = f.lines().map(|s| s.unwrap().trim().to_string()).collect();
    println!("done");

    loop {
        println!("{} options remaining", options.len());
        println!();

        println!("Computing best next guess...");
        let best_guess = if options.len() < 1_000 {
            let best_guess = mask::compute_best_guess(&options);
            println!("{}, score: {}", best_guess.0, best_guess.1);
            best_guess.0.to_string()
        } else {
            let subset: Vec<_> = options.choose_multiple(&mut thread_rng(), 500).collect();
            let best_guess = mask::compute_best_guess(&subset);
            println!(
                "{}, score: {} (based on 500 randomly-selected examples)",
                best_guess.0, best_guess.1
            );
            best_guess.0.to_string()
        };
        println!();

        for v in options.choose_multiple(&mut thread_rng(), 25).take(25) {
            println!("- {}", v);
        }
        if options.len() > 25 {
            println!("- ...");
        }

        if options.len() == 1 {
            return;
        }

        println!();
        let mut guess = prompt("Enter your guess (you can use s for ² and c for ³)");

        if guess.trim().is_empty() {
            println!("Using {} as the guess", best_guess);
            guess = best_guess;
        }

        let mask_txt = loop {
            let txt =
                prompt("Enter your mask (G or 2 for green; P or 1 for purple; B or 0 for black)");
            if !txt.is_empty() {
                break txt;
            }
        };

        if let Some(m) = mask::parse_mask_results(&guess, &mask_txt) {
            options.retain(|o| mask::matches_mask(o, &m));
        }
    }
}
