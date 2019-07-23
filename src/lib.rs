//! Defines structs for reading and writing bit by bit.
//! 
//! Author --- daniel.bechaz@gmail.com  
//! Last Moddified --- 2019-07-24

#![deny(missing_docs,)]
#![feature(const_fn,)]

mod read;
mod write;

pub use self::{read::*, write::*,};
