//! Defines structs for reading and writing bit by bit.
//! 
//! Author --- daniel.bechaz@gmail.com  
//! Last Moddified --- 2019-11-13

#![deny(missing_docs,)]
#![feature(const_fn,)]

mod bits;
mod read;
mod write;

pub use self::{bits::*, read::*, write::*,};
