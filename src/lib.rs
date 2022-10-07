#![feature(internal_output_capture)]

pub mod ast;
pub mod examples;
pub mod vm;

pub mod prelude {
    pub use crate::{ast::*, vm};
}
