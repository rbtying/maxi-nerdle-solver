//! An evaluator for Nerdle-style equations, based on the `nom` arithmetic
//! parser example.
//!
//! Nerdle supports the following types:
//!
//! - digits 0-9
//! - parentheses
//! - squares ² and cubes ³ (note: we support using `s` and `c`, respectively)
//! - multiplication * and division /
//! - addition + and subtraction -
//!
//! It also does its internal evaluation using rational numbers, not using integers!

use std::str::FromStr;

use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::char,
    character::complete::{digit1 as digit, space0 as space},
    combinator::map_res,
    error::ParseError,
    multi::fold_many0,
    sequence::{delimited, pair},
    IResult, Parser,
};
use num_rational::Rational32;
use num_traits::{checked_pow, CheckedAdd, CheckedDiv, CheckedMul, CheckedSub};

// We parse any expr surrounded by parens, ignoring all whitespaces around those
fn parens(i: &str) -> IResult<&str, Rational32> {
    delimited(space, delimited(tag("("), expr, tag(")")), space).parse(i)
}

// We transform an integer string into a Rational32, ignoring surrounding whitespaces
// We look for a digit suite, and try to convert it.
// If either str::from_utf8 or FromStr::from_str fail,
// we fallback to the parens parser defined above
fn factor(i: &str) -> IResult<&str, Rational32> {
    alt((
        map_res(delimited(space, digit, space), FromStr::from_str),
        parens,
    ))
    .parse(i)
}

// We apply any number of squares or cubes to a `factor`, which might be a
// parenthesized expression
fn exponent(i: &str) -> IResult<&str, Rational32> {
    let (i, init) = factor(i)?;
    fold_many0(
        alt((char('²'), char('s'), char('³'), char('c'))),
        move || Some(init),
        |acc, op: char| {
            acc.and_then(|acc| match op {
                '²' | 's' => checked_pow(acc, 2),
                '³' | 'c' => checked_pow(acc, 3),
                _ => unreachable!(),
            })
        },
    )
    .parse(i)
    .and_then(|(x, v)| {
        v.map(|v| (x, v)).ok_or_else(|| {
            nom::Err::Error(nom::error::Error::from_error_kind(
                i,
                nom::error::ErrorKind::Fail,
            ))
        })
    })
}

// We read an initial factor and for each time we find
// a * or / operator followed by another factor, we do
// the math by folding everything
fn term(i: &str) -> IResult<&str, Rational32> {
    let (i, init) = exponent(i)?;

    fold_many0(
        pair(alt((char('*'), char('/'))), exponent),
        move || Some(init),
        |acc, (op, val): (char, Rational32)| {
            acc.and_then(|acc| {
                if op == '*' {
                    acc.checked_mul(&val)
                } else {
                    acc.checked_div(&val)
                }
            })
        },
    )
    .parse(i)
    .and_then(|(x, v)| {
        v.map(|v| (x, v)).ok_or_else(|| {
            nom::Err::Error(nom::error::Error::from_error_kind(
                i,
                nom::error::ErrorKind::Fail,
            ))
        })
    })
}

fn expr(i: &str) -> IResult<&str, Rational32> {
    let (i, init) = term(i)?;

    fold_many0(
        pair(alt((char('+'), char('-'))), term),
        move || Some(init),
        |acc, (op, val): (char, Rational32)| {
            acc.and_then(|acc| {
                if op == '+' {
                    acc.checked_add(&val)
                } else {
                    acc.checked_sub(&val)
                }
            })
        },
    )
    .parse(i)
    .and_then(|(x, v)| {
        v.map(|v| (x, v)).ok_or_else(|| {
            nom::Err::Error(nom::error::Error::from_error_kind(
                i,
                nom::error::ErrorKind::Fail,
            ))
        })
    })
}

/// Evaluate the provided string, returning an integer result or an error.
pub fn eval(i: &str) -> Result<i32, anyhow::Error> {
    let (rem, v) = expr(i).map_err(|e| e.to_owned())?;
    if !rem.is_empty() {
        anyhow::bail!("string not fully evaluated: {} rem: {}", i, rem);
    }

    if !v.is_integer() {
        anyhow::bail!("did not get an integer result");
    }

    Ok(v.to_integer())
}

#[test]
fn test_evaluator() {
    assert_eq!(eval(" (  2 )").unwrap(), 2);
    assert_eq!(eval(" (  2 )").unwrap(), 2);
    assert_eq!(eval(" 2* (  3 + 4 ) ").unwrap(), 14);
    assert_eq!(eval("  2*2 / ( 5 - 1) + 3").unwrap(), 4);
    assert_eq!(eval("(5/4) * (4/5)").unwrap(), 1);
    assert_eq!(eval("0³/15+3²").unwrap(), 9);
    assert!(eval("4/5").is_err());
}
