extern crate boggle;

use boggle::{Dict, Grid};

fn main() {
    let mut dict = Dict::new();
    let words = include_str!("/usr/share/dict/british").split_whitespace();

    for word in words {
        dict.add(word);
    }

    let grid = "gbahwcaickoyelnh".parse::<Grid>().unwrap();
    let words = grid.words(&dict);

    for word in words {
        println!("{}", word);
    }
}
