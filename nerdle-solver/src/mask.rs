use std::collections::{HashMap, HashSet};

#[derive(Default, Debug)]
pub struct Masks {
    correct: HashSet<(usize, char)>,
    incorrect: HashSet<(usize, char)>,
    not_present: HashSet<(usize, char)>,
}

// Parse the provided string to an evaluation mask
//
// Correct location is represented by '2', 'C', 'c', 'G', 'g'
// Incorrect location is represented by '1', 'I', 'i', 'P', 'p'
// Not present is represented by '0', 'N', 'n', 'B', 'b', 'R', 'r', ' '
//
// Guesses will be normalized to use the square/cubed characters rather than the
// s and c characters.
pub fn parse_mask_results(guess: &str, mask: &str) -> Option<Masks> {
    let mut masks = Masks::default();

    if mask.chars().count() != guess.chars().count() {
        eprintln!(
            "Mask length doesn't match guess length! mask: {} guess: {}",
            mask, guess
        );
        return None;
    }

    for (idx, (m, c)) in mask.chars().zip(guess.chars()).enumerate() {
        match m {
            '2' | 'C' | 'c' | 'G' | 'g' => {
                masks.correct.insert((idx, normalize_power(c)));
            }
            '1' | 'I' | 'i' | 'P' | 'p' => {
                masks.incorrect.insert((idx, normalize_power(c)));
            }
            '0' | 'N' | 'n' | 'B' | 'b' | 'R' | 'r' | ' ' => {
                masks.not_present.insert((idx, normalize_power(c)));
            }
            _ => {
                eprintln!("Incorrect mask char '{}' in \"{}\"", c, mask);
                return None;
            }
        }
    }
    Some(masks)
}

fn normalize_power(c: char) -> char {
    match c {
        's' => '²',
        'c' => '³',
        x => x,
    }
}

pub fn matches_mask(guess: &str, mask: &Masks) -> bool {
    let chars = guess
        .chars()
        .enumerate()
        .collect::<HashSet<(usize, char)>>();

    if mask.not_present.intersection(&chars).next().is_some() {
        return false;
    }
    if mask.incorrect.intersection(&chars).next().is_some() {
        return false;
    }
    if mask.correct.intersection(&chars).count() < mask.correct.len() {
        return false;
    }

    let mut char_counts: HashMap<char, usize> = HashMap::new();
    for (idx, c) in guess.chars().enumerate() {
        if !mask.correct.contains(&(idx, c)) {
            *char_counts.entry(c).or_default() += 1;
        }
    }

    for (_, c) in &mask.incorrect {
        let ct = char_counts.entry(*c).or_default();
        if *ct == 0 {
            return false;
        }
        *ct -= 1;
    }
    for (_, c) in &mask.not_present {
        if let Some(ct) = char_counts.get(c) {
            if *ct > 0 {
                return false;
            }
        }
    }

    true
}
