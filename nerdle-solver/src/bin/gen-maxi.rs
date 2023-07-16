use std::fs::File;
use std::io::BufWriter;

use nerdle_solver::gen;

fn main() {
    let mut ct = 0;
    let mut f = BufWriter::new(File::create("maxi_nerdle.txt").unwrap());
    gen::gen(10, &mut gen::line_writer(&mut f, &mut ct), true);
    assert_eq!(ct, 2_177_736);
}
