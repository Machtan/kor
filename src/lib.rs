#![feature(unicode)]

extern crate std_unicode;
extern crate regex;
extern crate unicode_normalization;
#[macro_use]
extern crate lazy_static;
extern crate hangeul2;

#[macro_use]
mod common;
mod wordlist;
mod trie;
mod dict;
mod translate;

pub use wordlist::{Def, read_definitions, read_definitions_iter};
pub use dict::Dict;
pub use translate::{translate, translate_iter};
