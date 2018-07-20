extern crate boggle;

use std::env;
use std::fs::File;
use boggle::Dict;
use std::path::PathBuf;

fn main() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    let mut dict = Dict::new();
    let words = include_str!("/usr/share/dict/british");

    for word in words.split_whitespace() {
        dict.add(word)
    }

    let mut dict_file = File::create(out_dir.join("dict")).unwrap();
    dict.serialize_packed(&mut dict_file).unwrap();
}
