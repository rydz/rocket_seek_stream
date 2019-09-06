#![feature(seek_convenience)]

mod seekstream;
mod multipart;

pub use seekstream::{SeekStream, ReadSeek};
