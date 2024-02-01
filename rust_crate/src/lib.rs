pub use interface::*;

pub mod externs;
mod macros;

mod interface;
#[cfg(not(target_family = "wasm"))]
mod interface_os;
#[cfg(target_family = "wasm")]
mod interface_web;
