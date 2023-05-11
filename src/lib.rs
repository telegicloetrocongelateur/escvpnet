#![allow(incomplete_features)]
#![feature(generic_const_exprs)]
pub mod client;
pub mod command;
pub mod error;
pub mod header;
pub mod io;
pub mod packet;

pub use error::Error;

pub type Result<T> = std::result::Result<T, crate::Error>;

#[cfg(test)]
mod tests {
}
