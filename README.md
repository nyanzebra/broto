# broto
A byte encoder/decoder crate for easily converting structs to/from streams.

# Base type impls
The following types are already implemented with `Encode` and `Decode`:
```
()
(T,)
(T, U)
u8
u16
u32
u64
u128
usize
i8
i16
i32
i64
i128
isize
f32
f64
String
Option<T>
std::result::Result<T, E>
Vec<T>
```

As long as your `struct` and `enum` already use these type then no additional work is required. Otherwise you will need to add a custom implementation for `Encode` & `Decode` for everything to work.

# Examples
```rust
use broto::{Encode, Decode};

#[derive(Debug, PartialEq, Encode, Decode)]
struct Point {
    x: i32,
    y: i32,
}

#[derive(Debug, PartialEq, Encode, Decode)]
enum Shape {
    Circle { radius: u32 },
    Rectangle(u32, u32),
    Point(Point),
    Empty,
}
```
Would expand to this:
```rust
impl Encode for Point {
    fn encode<W>(&self, writer: &mut W) -> ::broto::Result<usize>
    where
        W: ::std::io::Write,
    {
        let mut total = 0usize;
        total += ::broto::Encode::encode(&self.x, writer)?;
        total += ::broto::Encode::encode(&self.y, writer)?;
        Ok(total)
    }
}
impl Decode for Point {
    fn decode<R>(reader: &mut R) -> ::broto::Result<Self>
    where
        Self: Sized,
        R: ::std::io::Read,
    {
        let x = <i32 as ::broto::Decode>::decode(reader)?;
        let y = <i32 as ::broto::Decode>::decode(reader)?;
        Ok(Point { x, y })
    }
}

impl Encode for Shape {
    fn encode<W>(&self, writer: &mut W) -> ::broto::Result<usize>
    where
        W: ::std::io::Write,
    {
        let mut total = 0usize;
        match self {
            Shape::Circle { radius } => {
                total += ::broto::Encode::encode(&(0u8 as u8), writer)?;
                total += ::broto::Encode::encode(radius, writer)?;
            }
            Shape::Rectangle(f0, f1) => {
                total += ::broto::Encode::encode(&(1u8 as u8), writer)?;
                total += ::broto::Encode::encode(f0, writer)?;
                total += ::broto::Encode::encode(f1, writer)?;
            }
            Shape::Point(f0) => {
                total += ::broto::Encode::encode(&(2u8 as u8), writer)?;
                total += ::broto::Encode::encode(f0, writer)?;
            }
            Shape::Empty => {
                total += ::broto::Encode::encode(&(3u8 as u8), writer)?;
            }
        }
        Ok(total)
    }
}
impl Decode for Shape {
    fn decode<R>(reader: &mut R) -> ::broto::Result<Self>
    where
        Self: Sized,
        R: ::std::io::Read,
    {
        let discriminant = <u8 as ::broto::Decode>::decode(reader)?;
        match discriminant {
            0u8 => {
                let radius = <u32 as ::broto::Decode>::decode(reader)?;
                Ok(Shape::Circle { radius })
            }
            1u8 => {
                let f0 = <u32 as ::broto::Decode>::decode(reader)?;
                let f1 = <u32 as ::broto::Decode>::decode(reader)?;
                Ok(Shape::Rectangle(f0, f1))
            }
            2u8 => {
                let f0 = <Point as ::broto::Decode>::decode(reader)?;
                Ok(Shape::Point(f0))
            }
            3u8 => Ok(Shape::Empty),
            other => {
                Err(::broto::Error::InvalidDiscriminant {
                    got: other,
                    max: 4usize,
                })
            }
        }
    }
}
```
