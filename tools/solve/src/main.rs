extern crate boggle;

use std::env::args;
use boggle::{Dict, Grid};

fn main() {
    let grid_description = args().nth(1).expect("grid description missing");
    let grid = grid_description.parse::<Grid>().unwrap();
    let mut dict = Dict::new();
    let words = include_str!("/usr/share/dict/british").split_whitespace();

    for word in words {
        dict.add(word);
    }

    let words = grid.words(&dict);

    for word in words {
        println!("{}", word);
    }
}
