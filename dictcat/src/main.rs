extern crate boggle;

use boggle::Dict;
use std::fs::File;
use std::env::args;

fn main() {
    let src_path = args().nth(1).expect("input dictionary missing");
    let mut file = File::open(&src_path).expect(&src_path);

    let dict = Dict::deserialize_packed(&mut file).expect("failed to read dictionary");
    let mut words: Vec<&str> = dict.words().collect();
    words.sort();

    for word in words {
        println!("{}", word);
    }
}
