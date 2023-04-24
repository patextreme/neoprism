pub mod proto {
    include!(concat!(env!("OUT_DIR"), "/proto.rs"));
}

pub mod crypto;
pub mod did;
pub mod dlt;
pub mod util;
