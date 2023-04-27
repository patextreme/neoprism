use crate::proto::atala_operation::Operation;
use bytes::{Bytes, BytesMut};
use prost::Message;

pub trait MessageExt {
    fn encode_to_bytes(&self) -> Result<Bytes, prost::EncodeError>;
}

impl<T: Message> MessageExt for T {
    fn encode_to_bytes(&self) -> Result<Bytes, prost::EncodeError> {
        let mut buf = BytesMut::with_capacity(self.encoded_len());
        self.encode(&mut buf)?;
        Ok(buf.freeze())
    }
}

pub trait OperationExt {
    fn is_create_operation(&self) -> bool;
}

impl OperationExt for Operation {
    fn is_create_operation(&self) -> bool {
        match self {
            Operation::CreateDid(_) => true,
            _ => false,
        }
    }
}
