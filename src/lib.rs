#![deny(rust_2018_idioms)]
// buggy lints
#![allow(clippy::useless_asref, clippy::explicit_auto_deref)]

pub mod ast;
pub mod environment;
pub mod eval;
pub mod lexer;
pub mod parser;
pub mod repl;
