//! Run with: cargo run --example async_roundtrip --features async
//!
//! Uses `futures::io::Cursor`, which implements `futures_io::AsyncWrite` /
//! `AsyncRead` in memory — no particular async runtime required. Any
//! runtime's I/O types work the same way as long as they implement the
//! `futures_io` traits (tokio types need `tokio_util::compat` to bridge).

use broto::{Decode, Encode};
use futures::executor::block_on;
use futures::io::Cursor;

#[derive(Debug, PartialEq, Encode, Decode)]
struct Point {
    x: i32,
    y: i32,
}

#[derive(Debug, PartialEq, Encode, Decode)]
enum Shape {
    Circle {
        radius: u32,
    },
    #[tag(100)]
    Rectangle(u32, u32),
    Point(Point),
    Empty,
}

fn main() -> broto::Result<()> {
    block_on(run())
}

async fn run() -> broto::Result<()> {
    let shapes = vec![
        Shape::Circle { radius: 7 },
        Shape::Rectangle(3, 4),
        Shape::Point(Point { x: -1, y: 2 }),
        Shape::Empty,
    ];

    let mut writer = Cursor::new(Vec::new());
    for shape in &shapes {
        shape.encode(&mut writer).await?;
    }
    let bytes = writer.into_inner();
    println!("encoded {} bytes: {bytes:?}", bytes.len());

    let mut reader = Cursor::new(bytes);
    let mut decoded = Vec::new();
    for _ in 0..shapes.len() {
        decoded.push(Shape::decode(&mut reader).await?);
    }

    assert_eq!(shapes, decoded);
    println!("round-trip OK: {decoded:?}");
    Ok(())
}
