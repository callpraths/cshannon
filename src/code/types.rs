/// core::result::Result alias with uniform error type.
///
/// All public functions from this package return results of this type.
pub type Result<R> = core::result::Result<R, String>;
