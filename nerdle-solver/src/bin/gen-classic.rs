use std::fs::File;
use std::io::BufWriter;

use nerdle_solver::gen;

fn main() {
    let mut ct = 0;
    let mut f = BufWriter::new(File::create("classic_nerdle.txt").unwrap());
    gen::gen(8, &mut gen::line_writer(&mut f, &mut ct), false);
    assert_eq!(ct, 18_115);
}
