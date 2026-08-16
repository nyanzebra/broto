//! Run with: cargo run --example sync_roundtrip --features sync

use broto::{Decode, DecodeExt as _, Encode};

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
    let shapes = vec![
        Shape::Circle { radius: 7 },
        Shape::Rectangle(3, 4),
        Shape::Point(Point { x: -1, y: 2 }),
        Shape::Empty,
    ];

    // Encode: Vec<u8> implements std::io::Write.
    let mut buf = Vec::new();
    for shape in &shapes {
        shape.encode(&mut buf)?;
    }
    println!("encoded {} bytes: {buf:?}", buf.len());

    // Decode: &[u8] implements std::io::Read.
    let mut cursor: &[u8] = &buf;
    let mut decoded = Vec::new();
    for msg in cursor.messages::<Shape>() {
        decoded.push(msg?);
    }

    assert_eq!(shapes, decoded);
    println!("round-trip OK: {decoded:?}");
    Ok(())
}
