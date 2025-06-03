pub trait DidOps {
    fn method(&self) -> &str;
    fn method_specific_id(&self) -> &str;
}
