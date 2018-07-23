#[macro_use] extern crate serde_derive;

pub mod client {
    pub mod message;
    pub use self::message::Message;
}

pub mod server {
    pub mod message;
    pub use self::message::Message;
}
