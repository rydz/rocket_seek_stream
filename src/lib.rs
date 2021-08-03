#![feature(seek_stream_len)]

//! This crate provides a seekable stream responder for rocket that will satisfy range requests.
//!
//! # Examples
//! see the examples directory for more.
//! 
//! ```no_run
//! #![feature(proc_macro_hygiene, decl_macro)]
//! #[macro_use]
//! extern crate rocket;
//! use rocket_seek_stream::SeekStream;
//!
//! #[get("/")]
//! fn home<'a>() -> std::io::Result<SeekStream<'a>> {
//!     SeekStream::from_path("kosmodrom.webm")
//! }
//! 
//! fn main() {
//!     rocket::Rocket::custom(
//!         rocket::Config::build(rocket::config::Environment::Development)
//!             .address("localhost")
//!             .port(8000)
//!             .finalize()
//!             .unwrap(),
//!     )
//!     .mount("/", routes![home])
//!    .launch();
//! }
//!
//! ```

mod multipart;
mod seekstream;

pub use seekstream::{ReadSeek, SeekStream};
