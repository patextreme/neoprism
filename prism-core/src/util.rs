use bytes::{Bytes, BytesMut};
use lazy_static::lazy_static;
use prost::Message;
use regex::Regex;
use std::{collections::HashSet, rc::Rc};

lazy_static! {
    static ref URI_FRAGMENT_RE: Regex =
        Regex::new(r"^([A-Za-z0-9\-._~!$&'()*+,;=:@/?]|%[0-9A-Fa-f]{2})*$").unwrap();
}

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

pub(crate) trait VecExt<T> {
    fn map_rc(self) -> Vec<Rc<T>>
    where
        T: 'static;
}

impl<T> VecExt<T> for Vec<T> {
    fn map_rc(self) -> Vec<Rc<T>>
    where
        T: 'static,
    {
        self.into_iter().map(Rc::new).collect()
    }
}

/// Check if the given slice contains unique items.
///
/// # Examples
/// ```
/// use prism_core::util::is_slice_unique;
/// assert_eq!(is_slice_unique(&[1, 2, 3]), true);
/// assert_eq!(is_slice_unique(&[1, 2, 2]), false);
/// assert_eq!(is_slice_unique(&[1, 1, 1]), false);
/// assert_eq!(is_slice_unique::<i32>(&[]), true);
/// ```
pub fn is_slice_unique<T>(items: &[T]) -> bool
where
    T: Eq + std::hash::Hash,
{
    let mut set = HashSet::new();
    items.iter().all(|x| set.insert(x))
}

/// Check if the given string is a valid URI fragment.
///
/// # Example
///
/// ```
/// use prism_core::util::is_uri_fragment;
///
/// assert_eq!(is_uri_fragment("hello"), true);
/// assert_eq!(is_uri_fragment("hello%20world"), true);
/// assert_eq!(is_uri_fragment("@123"), true);
/// assert_eq!(is_uri_fragment("+-*/"), true);
/// assert_eq!(is_uri_fragment(""), true);
///
/// assert_eq!(is_uri_fragment("hello world"), false);
/// assert_eq!(is_uri_fragment(" "), false);
/// assert_eq!(is_uri_fragment("hello%"), false);
/// assert_eq!(is_uri_fragment("hello%2"), false);
/// assert_eq!(is_uri_fragment("hello#"), false);
/// ```
pub fn is_uri_fragment(s: &str) -> bool {
    URI_FRAGMENT_RE.is_match(s)
}
