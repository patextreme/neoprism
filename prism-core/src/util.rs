use bytes::{Bytes, BytesMut};
use prost::Message;

pub trait MessageExt {
    fn encode_to_bytes(&self) -> Bytes;
}

impl<T: Message> MessageExt for T {
    fn encode_to_bytes(&self) -> Bytes {
        let mut buf = BytesMut::with_capacity(self.encoded_len());
        self.encode(&mut buf).unwrap();
        buf.freeze()
    }
}
