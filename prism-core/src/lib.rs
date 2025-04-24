// TODO: remove
#![feature(error_generic_member_access, split_array, error_reporter)]
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
pub mod repo;
pub mod utils;
