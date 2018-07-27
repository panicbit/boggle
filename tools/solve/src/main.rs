extern crate boggle;
extern crate dict;

use std::env::args;
use boggle::Grid;

use dict::DICT;

fn main() {
    let grid_description = args().nth(1).expect("grid description missing");
    let grid = grid_description.parse::<Grid>().unwrap();
    let words = grid.words(&DICT);

    for word in words {
        println!("{}", word);
    }
}
