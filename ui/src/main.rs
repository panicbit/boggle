#![feature(non_modrs_mods)]
extern crate failure;
extern crate boggle;
extern crate boggle_common;
#[macro_use] extern crate yew;
extern crate stdweb;

use failure::Error;
use yew::prelude::*;
use yew::format::{Binary, Text};

mod model;
use self::model::Model;

fn main() {
    yew::initialize();
    App::<Model>::new().mount_to_body();
    yew::run_loop();
}

impl From<Binary> for BinaryMessage {
    fn from(m: Binary) -> Self {
        BinaryMessage(m)
    }
}

impl From<Text> for BinaryMessage {
    fn from(m: Text) -> Self {
        BinaryMessage(m.map(Vec::from))
    }
}

struct BinaryMessage(Result<Vec<u8>, Error>);
