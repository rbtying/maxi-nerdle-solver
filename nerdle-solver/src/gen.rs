//! A helper to generate all valid Nerdle equations for a given length
//!
//! Based partially on Digital Trauma's recursive generator approach
//! https://codegolf.stackexchange.com/a/258767

use std::io::Write;

use crate::eval::eval;

/// Call `visitor` on all valid Nerdle equations which take exactly `slots`
/// slots.
///
/// Note that the generated string will use 's' and 'c' for square and cube
/// symbols, respectively.
///
/// if `extended` is true, generates with parentheses, squares, and cubes
pub fn gen(slots: usize, visitor: &mut dyn FnMut(&str), extended: bool) {
    let mut buf = vec![0; slots];
    gen_nz_digit(0, 0, &mut buf, visitor, extended);
    if extended {
        gen_open(0, 0, &mut buf, visitor);
    }
}

///  Helper function for a visitor that writes the output to the provided file,
///  keeping the count in `ct`.
pub fn line_writer<'a>(f: &'a mut impl Write, ct: &'a mut usize) -> impl FnMut(&str) + 'a {
    move |s| {
        for c in s.chars() {
            write!(
                f,
                "{}",
                match c {
                    's' => '²',
                    'c' => '³',
                    c => c,
                }
            )
            .unwrap();
        }
        writeln!(f).unwrap();
        if *ct % 10000 == 0 {
            eprintln!("{}: {}", ct, s);
        }
        *ct += 1;
    }
}

/// Try to insert a nonzero digit at `index`, and then recurse
fn gen_nz_digit(
    index: usize,
    depth: usize,
    buf: &mut [u8],
    visitor: &mut dyn FnMut(&str),
    extended: bool,
) {
    if index >= buf.len() - 2 {
        return;
    }

    for i in 1..10 {
        buf[index] = char::from_digit(i, 10).unwrap() as u8;
        try_gen_eq(index + 1, depth, buf, visitor);
        gen_digit(index + 1, depth, 1, buf, visitor, extended);
        gen_oper(index + 1, depth, buf, visitor, extended);
        if extended {
            gen_squared(index + 1, depth, buf, visitor);
            gen_cubed(index + 1, depth, buf, visitor);
            if depth > 0 {
                gen_close(index + 1, depth, buf, visitor);
            }
        }
    }
}

/// Try to insert a digit at `index`, and then recurse
fn gen_digit(
    index: usize,
    depth: usize,
    ndigits: usize,
    buf: &mut [u8],
    visitor: &mut dyn FnMut(&str),
    extended: bool,
) {
    if index >= buf.len() - 2 {
        return;
    }

    // There's no point generating numbers longer than half the available LHS,
    // since the resulting value won't fit on the RHS
    if ndigits >= (buf.len() - 2) / 2 {
        return;
    }
    for i in (1..10).chain(std::iter::once(0)) {
        buf[index] = char::from_digit(i, 10).unwrap() as u8;
        try_gen_eq(index + 1, depth, buf, visitor);
        gen_digit(index + 1, depth, ndigits + 1, buf, visitor, extended);
        gen_oper(index + 1, depth, buf, visitor, extended);
        if extended {
            gen_squared(index + 1, depth, buf, visitor);
            gen_cubed(index + 1, depth, buf, visitor);
            if depth > 0 {
                gen_close(index + 1, depth, buf, visitor);
            }
        }
    }
}

/// Try to insert an operator at `index`, and then recurse
fn gen_oper(
    index: usize,
    depth: usize,
    buf: &mut [u8],
    visitor: &mut dyn FnMut(&str),
    extended: bool,
) {
    if index > buf.len() - 3 {
        return;
    }
    for op in [b'-', b'+', b'*', b'/'] {
        buf[index] = op;
        gen_nz_digit(index + 1, depth, buf, visitor, extended);
        gen_open(index + 1, depth, buf, visitor);
    }
}

/// Try to insert a square at `index`, and then recurse. Use `s` rather than the
/// unicode square symbol so we use only one byte.
fn gen_squared(index: usize, depth: usize, buf: &mut [u8], visitor: &mut dyn FnMut(&str)) {
    if index > buf.len() - 2 {
        return;
    }
    buf[index] = b's';
    if index >= 3 {
        try_gen_eq(index + 1, depth, buf, visitor);
    }
    gen_oper(index + 1, depth, buf, visitor, true);
    if depth > 0 {
        gen_close(index + 1, depth, buf, visitor);
    }
}

/// Try to insert a cube at `index`, and then recurse. Use `s` rather than the
/// unicode cube symbol so we use only one byte.
fn gen_cubed(index: usize, depth: usize, buf: &mut [u8], visitor: &mut dyn FnMut(&str)) {
    if index > buf.len() - 2 {
        return;
    }
    buf[index] = b'c';
    if index >= 2 {
        try_gen_eq(index + 1, depth, buf, visitor);
    }
    gen_oper(index + 1, depth, buf, visitor, true);
    if depth > 0 {
        gen_close(index + 1, depth, buf, visitor);
    }
}

/// Try to insert an open parentheses at `index`, and then recurse
fn gen_open(index: usize, depth: usize, buf: &mut [u8], visitor: &mut dyn FnMut(&str)) {
    if index > buf.len() - 3 {
        return;
    }
    buf[index] = b'(';
    gen_nz_digit(index + 1, depth + 1, buf, visitor, true);
    gen_open(index + 1, depth + 1, buf, visitor);
}

/// Try to insert a close parentheses at `index`, and then recurse
fn gen_close(index: usize, depth: usize, buf: &mut [u8], visitor: &mut dyn FnMut(&str)) {
    debug_assert!(depth > 0);
    if index > buf.len() - 2 {
        return;
    }
    buf[index] = b')';
    try_gen_eq(index + 1, depth - 1, buf, visitor);
    gen_oper(index + 1, depth - 1, buf, visitor, true);
    gen_squared(index + 1, depth - 1, buf, visitor);
    gen_cubed(index + 1, depth - 1, buf, visitor);
    if depth - 1 > 0 {
        gen_close(index + 1, depth - 1, buf, visitor);
    }
}

/// Try to insert an equals sign at `index`, and then compute the value and call
/// `visitor` if it's the right size.
fn try_gen_eq(index: usize, depth: usize, buf: &mut [u8], visitor: &mut dyn FnMut(&str)) {
    if depth > 0 {
        return;
    }
    let expr = std::str::from_utf8(&buf[..index]).unwrap();
    if let Ok(v) = eval(expr) {
        if v < 0 {
            // Nerdle doesn't have negative-number solutions
            return;
        }
        let v_len = if v == 0 { 1 } else { (v.ilog10() + 1) as usize };
        if index + v_len + 1 == buf.len() {
            buf[index] = b'=';
            write!(&mut buf[index + 1..], "{}", v).unwrap();
            visitor(std::str::from_utf8(buf).unwrap());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::gen;

    #[test]
    fn test_gen() {
        let mut ct = 0;
        gen(
            6,
            &mut |_| {
                ct += 1;
            },
            true,
        );

        assert_eq!(ct, 404);
    }
}
