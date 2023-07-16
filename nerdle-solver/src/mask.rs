use std::collections::{BTreeSet, HashMap};

use rayon::prelude::*;

#[derive(Clone, Default, Debug, PartialEq, Eq, Hash)]
pub struct Masks {
    correct: BTreeSet<(usize, char)>,
    incorrect: BTreeSet<(usize, char)>,
    not_present: BTreeSet<(usize, char)>,
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
        .collect::<BTreeSet<(usize, char)>>();

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

pub fn score(guess: &str, truth: &str) -> Masks {
    let mut m = Masks::default();

    let mut truth_char_counts: HashMap<char, usize> = HashMap::new();
    for (idx, (g, t)) in guess.chars().zip(truth.chars()).enumerate() {
        if g == t {
            m.correct.insert((idx, g));
        } else {
            *truth_char_counts.entry(t).or_default() += 1;
        }
    }

    for (idx, g) in guess.chars().enumerate() {
        if m.correct.contains(&(idx, g)) {
            continue;
        }
        let v = truth_char_counts.entry(g).or_default();
        if *v > 0 {
            *v -= 1;
            m.incorrect.insert((idx, g));
        } else {
            m.not_present.insert((idx, g));
        }
    }

    m
}

/// Compute the guess which has the highest entropy in the corpus
pub fn compute_best_guess<T: AsRef<str> + Sync>(corpus: &[T]) -> (&str, f64) {
    corpus
        .par_iter()
        .map(|w| (w.as_ref(), compute_entropy(w.as_ref(), corpus)))
        .max_by(|a, b| a.1.total_cmp(&b.1))
        .unwrap()
}

// Compute the entropy of the given guess `x` against the corpus.
//
// Note that we don't subtract out the corpus.len().log2(), since that doesn't
// change the ordering of the entropies.
pub fn compute_entropy<T: AsRef<str>>(x: &str, corpus: &[T]) -> f64 {
    let mut masks: HashMap<Masks, usize> = HashMap::new();
    for v in corpus {
        let m = score(x, v.as_ref());
        *masks.entry(m).or_default() += 1;
    }

    let mut t = 0.;

    for (m, ct) in masks {
        let matching = corpus
            .iter()
            .filter(|c| matches_mask(c.as_ref(), &m))
            .count();
        let p = -(matching as f64).log2();

        t += ct as f64 * p;
    }
    t
}

#[cfg(test)]
mod tests {
    use super::{compute_best_guess, compute_entropy, score, Masks};

    #[test]
    fn test_score() {
        let m = score("abc", "cbd");
        assert_eq!(
            m,
            Masks {
                correct: [(1, 'b')].into_iter().collect(),
                incorrect: [(2, 'c')].into_iter().collect(),
                not_present: [(0, 'a')].into_iter().collect(),
            }
        )
    }

    #[test]
    fn test_entropy() {
        let e = compute_entropy("abc", &["abc", "abd", "aba"]);
        assert_eq!(e, -2.)
    }

    #[test]
    fn test_compute_best_guess() {
        let g = compute_best_guess(&["abc", "abd", "aba"]);
        assert_eq!(g, ("aba", -2.));
    }
}
