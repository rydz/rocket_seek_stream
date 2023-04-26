//! This crate provides a seekable stream responder for rocket that will satisfy range requests.
//!
//! # Examples
//! see the examples directory for more.
//!
//! ```no_run
//! use rocket::{get, launch, routes};
//! use rocket_seek_stream::SeekStream;
//!
//! #[get("/")]
//! fn home<'a>() -> std::io::Result<SeekStream<'a>> {
//!     SeekStream::from_path("kosmodrom.webm")
//! }
//!
//! #[launch]
//! fn rocket() -> _ {
//!     rocket::Rocket::build()
//!         .mount("/", routes![home])
//! }
//!
//! ```

mod multipart;
mod seekstream;

pub use seekstream::{ReadSeek, SeekStream};
