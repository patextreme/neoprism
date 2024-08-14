// TODO: remove
#![feature(error_generic_member_access, split_array)]
#![allow(dead_code)]

pub mod proto {
    include!(concat!(env!("OUT_DIR"), "/proto.rs"));
}

pub mod crypto;
pub mod did;
pub mod dlt;
pub mod error;
mod macros;
pub mod prelude;
pub mod protocol;
pub mod store;
pub mod utils;

pub use error::Error;
