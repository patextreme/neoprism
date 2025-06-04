#![feature(error_reporter)]

pub mod proto {
    include!(concat!(env!("OUT_DIR"), "/proto.rs"));
}

pub mod did;
pub mod dlt;
pub mod error;
mod macros;
pub mod prelude;
pub mod protocol;
pub mod repo;
pub mod utils;
