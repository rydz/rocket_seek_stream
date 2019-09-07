# rocket_seek_stream
A [Rocket](https://github.com/SergioBenitez/Rocket) responder for types implementing the `Read + Seek` traits, such as files and `std::io::Cursor`, that will respond to range requests with a 206 Partial Content response. The `Content-Type` can optionally be inferred by taking a sample of bytes from the beginning of the stream, or given manually. An `Accept-Ranges: bytes` header will be sent in all responses to notify browsers that range requests are supported for the resource.

This supports both single and multipart/byterange requests.
[https://tools.ietf.org/html/rfc7233](https://tools.ietf.org/html/rfc7233)

## Cargo.toml
Add this to your dependencies.

```
rocket_seek_stream = {git="https://github.com/rydz/rocket_seek_stream"}
```

## Examples
Serving a file from the disk

```rust
#![feature(proc_macro_hygiene, decl_macro)]
#[macro_use]
extern crate rocket;
use rocket_seek_stream::SeekStream;

#[get("/")]
fn home<'a>() -> std::io::Result<SeekStream<'a>> {
    SeekStream::from_path("kosmodrom.webm")
}
 
fn main() {
    rocket::Rocket::custom(
        rocket::Config::build(rocket::config::Environment::Development)
            .address("localhost")
            .port(8000)
            .finalize()
            .unwrap(),
    )
    .mount("/", routes![home])
   .launch();
}
```

Serving an in memory buffer
```rust
#[get("/")]
fn cursor<'a>() -> SeekStream<'a> {
    let bytes = &include_bytes!("./fly_me_to_the_moon.webm")[..];
    let len = bytes.len();
    let stream = std::io::Cursor::new(bytes);

    SeekStream::with_opts(stream, len as u64, "video/webm")
}
```

Use `cargo run --example server` to run the example. run `examples/download.sh` to download the media it depends on using [youtube-dl](https://github.com/ytdl-org/youtube-dl).