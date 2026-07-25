#![feature(prelude_import)]
//! Run with: cargo run --example sync_roundtrip --features sync
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
use broto::{Decode, Encode};
struct Point {
    x: i32,
    y: i32,
}
#[automatically_derived]
impl ::core::fmt::Debug for Point {
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        ::core::fmt::Formatter::debug_struct_field2_finish(
            f,
            "Point",
            "x",
            &self.x,
            "y",
            &&self.y,
        )
    }
}
#[automatically_derived]
impl ::core::marker::StructuralPartialEq for Point {}
#[automatically_derived]
impl ::core::cmp::PartialEq for Point {
    #[inline]
    fn eq(&self, other: &Point) -> bool {
        self.x == other.x && self.y == other.y
    }
}
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
enum Shape {
    Circle { radius: u32 },
    Rectangle(u32, u32),
    Point(Point),
    Empty,
}
#[automatically_derived]
impl ::core::fmt::Debug for Shape {
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        match self {
            Shape::Circle { radius: __self_0 } => {
                ::core::fmt::Formatter::debug_struct_field1_finish(
                    f,
                    "Circle",
                    "radius",
                    &__self_0,
                )
            }
            Shape::Rectangle(__self_0, __self_1) => {
                ::core::fmt::Formatter::debug_tuple_field2_finish(
                    f,
                    "Rectangle",
                    __self_0,
                    &__self_1,
                )
            }
            Shape::Point(__self_0) => {
                ::core::fmt::Formatter::debug_tuple_field1_finish(f, "Point", &__self_0)
            }
            Shape::Empty => ::core::fmt::Formatter::write_str(f, "Empty"),
        }
    }
}
#[automatically_derived]
impl ::core::marker::StructuralPartialEq for Shape {}
#[automatically_derived]
impl ::core::cmp::PartialEq for Shape {
    #[inline]
    fn eq(&self, other: &Shape) -> bool {
        let __self_discr = ::core::intrinsics::discriminant_value(self);
        let __arg1_discr = ::core::intrinsics::discriminant_value(other);
        __self_discr == __arg1_discr
            && match (self, other) {
                (
                    Shape::Circle { radius: __self_0 },
                    Shape::Circle { radius: __arg1_0 },
                ) => __self_0 == __arg1_0,
                (
                    Shape::Rectangle(__self_0, __self_1),
                    Shape::Rectangle(__arg1_0, __arg1_1),
                ) => __self_0 == __arg1_0 && __self_1 == __arg1_1,
                (Shape::Point(__self_0), Shape::Point(__arg1_0)) => __self_0 == __arg1_0,
                _ => true,
            }
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
fn main() -> broto::Result<()> {
    let shapes = ::alloc::boxed::box_assume_init_into_vec_unsafe(
        ::alloc::intrinsics::write_box_via_move(
            ::alloc::boxed::Box::new_uninit(),
            [
                Shape::Circle { radius: 7 },
                Shape::Rectangle(3, 4),
                Shape::Point(Point { x: -1, y: 2 }),
                Shape::Empty,
            ],
        ),
    );
    let mut buf = Vec::new();
    for shape in &shapes {
        shape.encode(&mut buf)?;
    }
    {
        ::std::io::_print(format_args!("encoded {0} bytes: {1:?}\n", buf.len(), buf));
    };
    let mut cursor: &[u8] = &buf;
    let mut decoded = Vec::new();
    for _ in 0..shapes.len() {
        decoded.push(Shape::decode(&mut cursor)?);
    }
    match (&shapes, &decoded) {
        (left_val, right_val) => {
            if !(*left_val == *right_val) {
                let kind = ::core::panicking::AssertKind::Eq;
                ::core::panicking::assert_failed(
                    kind,
                    &*left_val,
                    &*right_val,
                    ::core::option::Option::None,
                );
            }
        }
    };
    {
        ::std::io::_print(format_args!("round-trip OK: {0:?}\n", decoded));
    };
    Ok(())
}
