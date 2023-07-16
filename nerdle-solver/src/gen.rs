//! A helper to generate all valid Nerdle equations for a given length

use std::io::Write;

use crate::eval::eval;

pub fn gen(buf: &mut [u8], visitor: &mut dyn FnMut(&str)) {
    gen_nz_digit(0, 0, buf, visitor);
    gen_open(0, 0, buf, visitor);
}

fn gen_nz_digit(index: usize, depth: usize, buf: &mut [u8], visitor: &mut dyn FnMut(&str)) {
    if index >= buf.len() - 2 {
        return;
    }

    for i in 1..10 {
        buf[index] = char::from_digit(i, 10).unwrap() as u8;
        try_gen_eq(index + 1, depth, buf, visitor);
        gen_digit(index + 1, depth, 1, buf, visitor);
        gen_oper(index + 1, depth, buf, visitor);
        gen_squared(index + 1, depth, buf, visitor);
        gen_cubed(index + 1, depth, buf, visitor);
        if depth > 0 {
            gen_close(index + 1, depth, buf, visitor);
        }
    }
}

fn gen_digit(
    index: usize,
    depth: usize,
    ndigits: usize,
    buf: &mut [u8],
    visitor: &mut dyn FnMut(&str),
) {
    if index >= buf.len() - 2 {
        return;
    }
    if ndigits >= (buf.len() - 2) / 2 {
        return;
    }
    for i in (1..10).chain(std::iter::once(0)) {
        buf[index] = char::from_digit(i, 10).unwrap() as u8;
        try_gen_eq(index + 1, depth, buf, visitor);
        gen_digit(index + 1, depth, ndigits + 1, buf, visitor);
        gen_oper(index + 1, depth, buf, visitor);
        gen_squared(index + 1, depth, buf, visitor);
        gen_cubed(index + 1, depth, buf, visitor);
        if depth > 0 {
            gen_close(index + 1, depth, buf, visitor);
        }
    }
}

fn gen_oper(index: usize, depth: usize, buf: &mut [u8], visitor: &mut dyn FnMut(&str)) {
    if index > buf.len() - 3 {
        return;
    }
    for op in [b'-', b'+', b'*', b'/'] {
        buf[index] = op;
        gen_nz_digit(index + 1, depth, buf, visitor);
        gen_open(index + 1, depth, buf, visitor);
    }
}

fn gen_squared(index: usize, depth: usize, buf: &mut [u8], visitor: &mut dyn FnMut(&str)) {
    if index > buf.len() - 2 {
        return;
    }
    buf[index] = b's';
    if index >= 3 {
        try_gen_eq(index + 1, depth, buf, visitor);
    }
    gen_oper(index + 1, depth, buf, visitor);
    if depth > 0 {
        gen_close(index + 1, depth, buf, visitor);
    }
}

fn gen_cubed(index: usize, depth: usize, buf: &mut [u8], visitor: &mut dyn FnMut(&str)) {
    if index > buf.len() - 2 {
        return;
    }
    buf[index] = b'c';
    if index >= 2 {
        try_gen_eq(index + 1, depth, buf, visitor);
    }
    gen_oper(index + 1, depth, buf, visitor);
    if depth > 0 {
        gen_close(index + 1, depth, buf, visitor);
    }
}

fn gen_open(index: usize, depth: usize, buf: &mut [u8], visitor: &mut dyn FnMut(&str)) {
    if index > buf.len() - 3 {
        return;
    }
    buf[index] = b'(';
    gen_nz_digit(index + 1, depth + 1, buf, visitor);
    gen_open(index + 1, depth + 1, buf, visitor);
}

fn gen_close(index: usize, depth: usize, buf: &mut [u8], visitor: &mut dyn FnMut(&str)) {
    debug_assert!(depth > 0);
    if index > buf.len() - 2 {
        return;
    }
    buf[index] = b')';
    try_gen_eq(index + 1, depth - 1, buf, visitor);
    gen_oper(index + 1, depth - 1, buf, visitor);
    gen_squared(index + 1, depth - 1, buf, visitor);
    gen_cubed(index + 1, depth - 1, buf, visitor);
    if depth - 1 > 0 {
        gen_close(index + 1, depth - 1, buf, visitor);
    }
}

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
        drop(expr);
        if index + v_len + 1 == buf.len() {
            buf[index] = b'=';
            write!(&mut buf[index + 1..], "{}", v).unwrap();
            visitor(std::str::from_utf8(&buf).unwrap());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::gen_nz_digit;

    #[test]
    fn test_gen() {
        let mut buf = vec![0; 10];
        let mut ct = 0;
        gen_nz_digit(0, 0, &mut buf, &mut |s| {
            if ct % 100000 == 0 {
                eprintln!("{}: {}", ct, s);
            }
            ct += 1;
        });

        assert_eq!(ct, 0);
    }
}
