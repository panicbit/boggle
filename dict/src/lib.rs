extern crate boggle;
#[macro_use] extern crate lazy_static;

use boggle::Dict;

lazy_static! {
    pub static ref DICT: Dict = {
        let mut dict = include_bytes!(concat!(env!("OUT_DIR"), "/dict")).as_ref();
        Dict::deserialize_packed(&mut dict).unwrap()
    };
}
