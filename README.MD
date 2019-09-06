# rocket_seek_stream
A [Rocket](https://github.com/SergioBenitez/Rocket) responder for types implementing the `Read + Seek` traits, such as files and `std::io::Cursor`, that will respond to range requests. The `Content-Type` can optionally be inferred by taking a sample of bytes from the beginning of the stream, or given manually.

## Cargo.toml
Add this to your dependencies.

```
rocket_seek_stream = {git="https://github.com/rydz/rocket_seek_stream"}
```

## Examples

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

Use `cargo run --example server` to run the example. run `examples/download.sh` to download the media it depends on using [youtube-dl](https://github.com/ytdl-org/youtube-dl).