mod eval;
mod gen;

fn main() {
    let mut buf = vec![0; 10];
    let mut ct = 0;
    gen::gen(&mut buf, &mut |s| {
        if ct % 100000 == 0 {
            eprintln!("{}: {}", ct, s);
        }
        ct += 1;
    });

    assert_eq!(ct, 0);
}
