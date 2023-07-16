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
//! It also does its internal evaluation using rational numbers when necessary,
//! since Nerdle permits intermediate fractions during evaluation.

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

fn make_err(i: &str, e: ComputeError) -> nom::Err<nom::error::Error<&str>> {
    nom::Err::Failure(nom::error::Error::from_error_kind(
        i,
        match e {
            ComputeError::ComputeError => nom::error::ErrorKind::Fail,
            ComputeError::NonIntegerDivision | ComputeError::NonIntegerResult => {
                nom::error::ErrorKind::Float
            }
        },
    ))
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum ComputeError {
    ComputeError,
    NonIntegerDivision,
    NonIntegerResult,
}

trait Val: Sized + Copy + std::fmt::Debug {
    fn add(self, other: Self) -> Result<Self, ComputeError>;
    fn sub(self, other: Self) -> Result<Self, ComputeError>;
    fn mul(self, other: Self) -> Result<Self, ComputeError>;
    fn div(self, other: Self) -> Result<Self, ComputeError>;
    fn pow(self, pow: usize) -> Result<Self, ComputeError>;
    fn to_integer(self) -> Option<i32>;
    fn from_integer(i: i32) -> Self;
}

impl Val for i32 {
    fn add(self, other: Self) -> Result<Self, ComputeError> {
        self.checked_add(other).ok_or(ComputeError::ComputeError)
    }
    fn sub(self, other: Self) -> Result<Self, ComputeError> {
        self.checked_sub(other).ok_or(ComputeError::ComputeError)
    }
    fn mul(self, other: Self) -> Result<Self, ComputeError> {
        self.checked_mul(other).ok_or(ComputeError::ComputeError)
    }
    fn div(self, other: Self) -> Result<Self, ComputeError> {
        if other == 0 {
            Err(ComputeError::ComputeError)
        } else if self % other == 0 {
            self.checked_div(other).ok_or(ComputeError::ComputeError)
        } else {
            Err(ComputeError::NonIntegerDivision)
        }
    }
    fn pow(self, pow: usize) -> Result<Self, ComputeError> {
        checked_pow(self, pow).ok_or(ComputeError::ComputeError)
    }
    fn to_integer(self) -> Option<i32> {
        Some(self)
    }
    fn from_integer(i: i32) -> Self {
        i
    }
}

impl Val for Rational32 {
    fn add(self, other: Self) -> Result<Self, ComputeError> {
        self.checked_add(&other).ok_or(ComputeError::ComputeError)
    }
    fn sub(self, other: Self) -> Result<Self, ComputeError> {
        self.checked_sub(&other).ok_or(ComputeError::ComputeError)
    }
    fn mul(self, other: Self) -> Result<Self, ComputeError> {
        self.checked_mul(&other).ok_or(ComputeError::ComputeError)
    }
    fn div(self, other: Self) -> Result<Self, ComputeError> {
        self.checked_div(&other).ok_or(ComputeError::ComputeError)
    }
    fn pow(self, pow: usize) -> Result<Self, ComputeError> {
        checked_pow(self, pow).ok_or(ComputeError::ComputeError)
    }
    fn to_integer(self) -> Option<i32> {
        if Rational32::is_integer(&self) {
            Some(Rational32::to_integer(&self))
        } else {
            None
        }
    }
    fn from_integer(i: i32) -> Self {
        Rational32::from_integer(i)
    }
}

// We parse any expr surrounded by parens, ignoring all whitespaces around those
fn parens<V: Val>(i: &str) -> IResult<&str, V> {
    delimited(space, delimited(tag("("), expr, tag(")")), space).parse(i)
}

// We transform an integer string into a Rational32, ignoring surrounding whitespaces
// We look for a digit suite, and try to convert it.
// If either str::from_utf8 or FromStr::from_str fail,
// we fallback to the parens parser defined above
fn factor<V: Val>(i: &str) -> IResult<&str, V> {
    alt((
        map_res(delimited(space, digit, space), |s| {
            i32::from_str(s).map(|v| V::from_integer(v))
        }),
        parens,
    ))
    .parse(i)
}

// We apply any number of squares or cubes to a `factor`, which might be a
// parenthesized expression
fn exponent<V: Val>(i: &str) -> IResult<&str, V> {
    let (i, init) = factor(i)?;
    fold_many0(
        alt((char('²'), char('s'), char('³'), char('c'))),
        move || Ok(init),
        |acc, op: char| {
            acc.and_then(|acc: V| match op {
                '²' | 's' => acc.pow(2),
                '³' | 'c' => acc.pow(3),
                _ => unreachable!(),
            })
        },
    )
    .parse(i)
    .and_then(|(x, v)| v.map(|v| (x, v)).map_err(|e| make_err(i, e)))
}

// We read an initial factor and for each time we find
// a * or / operator followed by another factor, we do
// the math by folding everything
fn term<V: Val>(i: &str) -> IResult<&str, V> {
    let (i, init) = exponent(i)?;

    fold_many0(
        pair(alt((char('*'), char('/'))), exponent),
        move || Ok(init),
        |acc, (op, val): (char, V)| {
            acc.and_then(|acc: V| {
                if op == '*' {
                    acc.mul(val)
                } else {
                    acc.div(val)
                }
            })
        },
    )
    .parse(i)
    .and_then(|(x, v)| v.map(|v| (x, v)).map_err(|e| make_err(i, e)))
}

fn expr<V: Val>(i: &str) -> IResult<&str, V> {
    let (i, init) = term(i)?;

    fold_many0(
        pair(alt((char('+'), char('-'))), term),
        move || Ok(init),
        |acc, (op, val): (char, V)| {
            acc.and_then(|acc: V| {
                if op == '+' {
                    acc.add(val)
                } else {
                    acc.sub(val)
                }
            })
        },
    )
    .parse(i)
    .and_then(|(x, v)| v.map(|v| (x, v)).map_err(|e| make_err(i, e)))
}

/// Evaluate the provided string, returning an integer result or an error.
pub fn eval(i: &str) -> Result<i32, nom::Err<nom::error::Error<&str>>> {
    match expr::<i32>(i) {
        Ok((rem, v)) if rem.is_empty() => Ok(v),
        Err(nom::Err::Failure(f)) if f.code == nom::error::ErrorKind::Float => {
            let (rem, v) = expr::<Rational32>(i)?;
            if !rem.is_empty() {
                Err(make_err(i, ComputeError::ComputeError))?
            }

            v.to_integer()
                .ok_or_else(|| make_err(i, ComputeError::NonIntegerResult))
        }
        Ok(_) => Err(make_err(i, ComputeError::ComputeError)),
        Err(e) => Err(e),
    }
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
