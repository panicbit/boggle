extern crate boggle;

use boggle::Dict;
use std::fs::File;
use std::env::args;
use std::io::{BufReader, BufRead};

fn main() {
    let src_path = args().nth(1).expect("missing path to word list");
    let dst_path = args().nth(2).expect("missing path to output");
    let input = File::open(&src_path).expect(&src_path);
    let input = BufReader::new(input);
    let mut output = File::create(&dst_path).expect(&dst_path);
    let mut dict = Dict::new();

    for word in input.lines() {
        let word = word.expect("could not read word from word list");
        let word = word.trim();
        
        dict.add(word);
    }

    dict.serialize_packed(&mut output).expect("failed to write dict");
}
