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
                    max: 3u8,
                })
            }
        }
    }
}
```

# Custom discriminants: `#[tag(N)]`
By default, each enum variant's discriminant is just its position in the enum, 0-indexed — that's what the `Shape` example above shows: `Circle` is `0`, `Rectangle` is `1`, `Point` is `2`, `Empty` is `3`.

If you need explicit control over the discriminant instead — interop with another wire format's existing values, keeping a variant's byte value stable even if you reorder or insert variants later, reserving specific values, etc. — tag a variant directly:
```rust
#[derive(Debug, PartialEq, Encode, Decode)]
enum Status {
    Ok,
    #[tag(100)]
    Retry,
    Failed,
}
```

- `Ok` has no `#[tag(...)]`, so it keeps its default: its position, `0`.
- `Retry` is explicitly `100`.
- `Failed` has no `#[tag(...)]` either — it still gets *its own* position, `2`, regardless of what `Retry` was tagged. Tags don't shift the numbering for variants that come after them, and untagged variants never need to know or care what nearby variants are tagged.

The discriminant written to the wire for each variant is `u8` (`0..=255`), same as untagged variants always were. Two things are checked at compile time, so a mistake here is a build failure, not something that surfaces later as a decode error:
- A `#[tag(...)]` value that doesn't fit in a `u8` fails to compile.
- Two variants ending up with the same discriminant — whether from two explicit tags, or a tag colliding with another variant's default position — fails to compile, naming both variants in the error.
