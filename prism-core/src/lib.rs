#![allow(unused_variables, unreachable_code, dead_code)] // TODO: remove

pub mod proto {
    include!(concat!(env!("OUT_DIR"), "/proto.rs"));
}

pub mod crypto;
pub mod did;
pub mod dlt;
pub mod prelude;
pub mod protocol;
pub mod store;
pub mod util;
