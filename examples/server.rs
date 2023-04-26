#![feature(proc_macro_hygiene, decl_macro)]

/// Run download-videos.sh to download the videos required to run this example
/// It depends on yt-dlp.
/// 
/// Use `cargo run --example server` then navigate to [localhost:8000](http://localhost:8000)
/// in your browser.
/// 
/// Change the path between "/"", "from_path", and "long" to see the different examples in action.

#[macro_use]
extern crate rocket;
use rocket_seek_stream::SeekStream;

// stream from an in memory buffer
#[get("/memory")]
fn hello<'a>() -> SeekStream<'a> {
    let bytes = &include_bytes!("./cruel_angels_thesis.webm")[..];
    let len = bytes.len();
    let stream = std::io::Cursor::new(bytes);

    SeekStream::with_opts(stream, len as u64, "video/webm")
}

// stream from a given filepath
#[get("/from_path")]
fn from_path<'a>() -> std::io::Result<SeekStream<'a>> {
    SeekStream::from_path("fly_me_to_the_moon.webm")
}

// some long media
#[get("/long")]
fn long<'a>() -> std::io::Result<SeekStream<'a>> {
    SeekStream::from_path("tari_tari.webm")
}

// some longer media
#[get("/")]
fn longer<'a>() -> std::io::Result<SeekStream<'a>> {
    SeekStream::from_path("ison.webm")
}

fn main() {
    rocket::Rocket::custom(
        rocket::Config::build(rocket::config::Environment::Development)
            .address("localhost")
            .port(8000)
            .finalize()
            .unwrap(),
    )
    .mount("/", routes![hello, from_path, long, longer])
    .launch();
}
