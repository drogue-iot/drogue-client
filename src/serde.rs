use base64_serde::base64_serde_type;

base64_serde_type!(pub Base64Standard, base64::STANDARD);

pub(crate) fn is_default<T: Default + PartialEq>(t: &T) -> bool {
    t == &T::default()
}
