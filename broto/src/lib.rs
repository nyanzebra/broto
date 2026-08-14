#[cfg(all(feature = "async", not(feature = "sync")))]
mod r#async;
#[cfg(all(feature = "async", not(feature = "sync")))]
pub use r#async::*;
#[cfg(all(feature = "sync", not(feature = "async")))]
mod sync;
#[cfg(all(feature = "sync", not(feature = "async")))]
pub use sync::*;

/// `broto` error types.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// On an invalid discriminant, decoding returns `Error::InvalidDiscriminant(discriminant)`,
    /// where `discriminant` is the invalid discriminant value.
    ///
    /// What is considered valid depends on the `#[tag(...)]` attributes used on the enum.
    ///
    /// For example:
    ///
    /// ```ignore
    /// #[derive(broto::Encode, broto::Decode)]
    /// enum Shape {
    ///   Circle,
    ///   Square,
    ///   Triangle,
    ///   Rhombus,
    /// }
    /// ```
    ///
    /// `Shape` (no tags, discriminants `0..=3`), the range is `0..=3`.
    ///
    /// With a tagged `enum`:
    /// ```ignore
    /// #[derive(broto::Encode, broto::Decode)]
    /// enum Status {
    ///   Ok,
    ///   #[tag(100)]
    ///   Retry,
    ///   Failed,
    /// }
    /// ```
    /// `Status` above has (`Ok = 0`, `Retry = 100`, `Failed = 2`),
    /// any value outside this range is considered invalid.
    #[error("Invalid discriminant '{0}'")]
    InvalidDiscriminant(u8),

    /// IO errors are returned as `Error::IO(std::io::Error)`.
    #[error("IO '{0}'")]
    IO(#[from] std::io::Error),
}

/// A convenience type alias for `std::result::Result<T, Error>`.
/// Where Error is [`Error`].
pub type Result<T> = std::result::Result<T, Error>;
