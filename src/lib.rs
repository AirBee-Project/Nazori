mod error;
mod formats;

#[cfg(target_arch = "wasm32")]
pub mod wasm;

#[doc(inline)]
pub use error::Error;
pub use formats::plateau;
