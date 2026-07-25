#[cfg(all(feature = "async", not(feature = "sync")))]
mod r#async;
#[cfg(all(feature = "async", not(feature = "sync")))]
pub use r#async::*;
#[cfg(all(feature = "sync", not(feature = "async")))]
mod sync;
#[cfg(all(feature = "sync", not(feature = "async")))]
pub use sync::*;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Invalid discriminant '{got}' (max: '{max}')")]
    InvalidDiscriminant { got: u8, max: usize },

    #[error("IO '{0}'")]
    IO(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, Error>;
