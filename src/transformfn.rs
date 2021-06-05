#[derive(Clone, Eq)]
pub struct TransformFn(pub(crate) Box<fn(String) -> String>);

impl std::fmt::Debug for TransformFn {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "<TransformFn>")
    }
}

/// **Huge caveat**: define _all_ transforms to be equal since we can't inspect what they're going to do.
/// This allows us to continue using `PartialEq` with [Generator]
impl PartialEq for TransformFn {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}
