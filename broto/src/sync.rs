use std::io::{Read, Write};

pub use broto_derive::{Decode, Encode};

use crate::Result;

pub trait Encode {
    fn encode<W>(&self, writer: &mut W) -> Result<usize>
    where
        W: Write;
}

pub trait Decode {
    fn decode<R>(reader: &mut R) -> Result<Self>
    where
        Self: Sized,
        R: Read;
}

mod unit {

    use std::io::{Read, Write};

    use crate::{
        Result,
        sync::{Decode, Encode},
    };

    impl Decode for () {
        fn decode<R>(_reader: &mut R) -> Result<Self>
        where
            R: Read,
        {
            Ok(())
        }
    }

    impl Encode for () {
        fn encode<W>(&self, _writer: &mut W) -> Result<usize>
        where
            W: Write,
        {
            Ok(0)
        }
    }
}

mod tuple {
    use std::io::{Read, Write};

    use crate::{
        Result,
        sync::{Decode, Encode},
    };

    impl<T> Decode for (T,)
    where
        T: Decode,
    {
        fn decode<R>(reader: &mut R) -> Result<Self>
        where
            R: Read,
        {
            let t = T::decode(reader)?;
            Ok((t,))
        }
    }

    impl<T> Encode for (T,)
    where
        T: Encode,
    {
        fn encode<W>(&self, writer: &mut W) -> Result<usize>
        where
            W: Write,
        {
            let (t,) = self;
            t.encode(writer)
        }
    }

    impl<T, U> Decode for (T, U)
    where
        T: Decode,
        U: Decode,
    {
        fn decode<R>(reader: &mut R) -> Result<Self>
        where
            R: Read,
        {
            let t = T::decode(reader)?;
            let u = U::decode(reader)?;
            Ok((t, u))
        }
    }

    impl<T, U> Encode for (T, U)
    where
        T: Encode,
        U: Encode,
    {
        fn encode<W>(&self, writer: &mut W) -> Result<usize>
        where
            W: Write,
        {
            let (t, u) = self;
            t.encode(writer)?;
            u.encode(writer)
        }
    }
}

mod numbers {

    macro_rules! decode {
        ($t:ty) => {
            impl Decode for $t {
                fn decode<R>(reader: &mut R) -> Result<Self>
                where
                    R: Read,
                {
                    let mut buf = [0u8; std::mem::size_of::<$t>()];
                    reader.read_exact(&mut buf)?;
                    Ok(<$t>::from_be_bytes(buf))
                }
            }
        };
    }

    macro_rules! encode {
        ($t:ty) => {
            impl Encode for $t {
                fn encode<W>(&self, writer: &mut W) -> Result<usize>
                where
                    W: Write,
                {
                    writer.write_all(&self.to_be_bytes())?;
                    Ok(std::mem::size_of::<$t>())
                }
            }
        };
    }

    mod unsigned {

        use std::io::{Read, Write};

        use crate::{
            Result,
            sync::{Decode, Encode},
        };

        decode!(u8);
        decode!(u16);
        decode!(u32);
        decode!(u64);
        decode!(u128);
        decode!(usize);

        encode!(u8);
        encode!(u16);
        encode!(u32);
        encode!(u64);
        encode!(u128);
        encode!(usize);
    }

    mod signed {

        use std::io::{Read, Write};

        use crate::{
            Result,
            sync::{Decode, Encode},
        };

        decode!(i8);
        decode!(i16);
        decode!(i32);
        decode!(i64);
        decode!(i128);
        decode!(isize);

        encode!(i8);
        encode!(i16);
        encode!(i32);
        encode!(i64);
        encode!(i128);
        encode!(isize);
    }

    mod floats {

        use std::io::{Read, Write};

        use crate::{
            Result,
            sync::{Decode, Encode},
        };

        decode!(f32);
        decode!(f64);

        encode!(f32);
        encode!(f64);
    }
}

mod string {

    use std::io::{Read, Write};

    use crate::{
        Result,
        sync::{Decode, Encode},
    };

    impl Decode for String {
        fn decode<R>(reader: &mut R) -> Result<Self>
        where
            R: Read,
        {
            let mut buf = [0u8; 2];
            reader.read_exact(&mut buf)?;
            let str_len = u16::from_be_bytes(buf) as usize;
            let mut bytes = vec![0u8; str_len];
            reader.read_exact(&mut bytes)?;
            let str = String::from_utf8_lossy(&bytes);
            Ok(str.to_string())
        }
    }

    impl Encode for String {
        fn encode<W>(&self, buf: &mut W) -> Result<usize>
        where
            W: Write,
        {
            let bytes = self.as_bytes();
            let len = (bytes.len() as u16).to_be_bytes();
            buf.write_all(&len)?;
            buf.write_all(bytes)?;
            Ok(2 + bytes.len())
        }
    }
}

mod options {

    use std::io::{Read, Write};

    use crate::{
        Result,
        sync::{Decode, Encode},
    };

    impl<T> Decode for Option<T>
    where
        T: Decode,
    {
        fn decode<R>(reader: &mut R) -> Result<Self>
        where
            R: Read,
        {
            let mut buf = [0u8; 1];
            reader.read_exact(&mut buf)?;
            if buf[0] == 0 {
                Ok(None)
            } else {
                let value = T::decode(reader)?;
                Ok(Some(value))
            }
        }
    }

    impl<T> Encode for Option<T>
    where
        T: Encode,
    {
        fn encode<W>(&self, buf: &mut W) -> Result<usize>
        where
            W: Write,
        {
            match self {
                None => {
                    buf.write_all(&[0])?;
                    Ok(1)
                }
                Some(value) => {
                    buf.write_all(&[1])?;
                    value.encode(buf)
                }
            }
        }
    }
}

mod result {

    use std::io::{Read, Write};

    use crate::{
        Result,
        sync::{Decode, Encode},
    };

    impl<T, E> Decode for std::result::Result<T, E>
    where
        T: Decode,
        E: Decode,
    {
        fn decode<R>(reader: &mut R) -> Result<Self>
        where
            R: Read,
        {
            let mut buf = [0u8; 1];
            reader.read_exact(&mut buf)?;
            if buf[0] == 0 {
                let value = T::decode(reader)?;
                Ok(Ok(value))
            } else {
                let value = E::decode(reader)?;
                Ok(Err(value))
            }
        }
    }

    impl<T, E> Encode for std::result::Result<T, E>
    where
        T: Encode,
        E: Encode,
    {
        fn encode<W>(&self, buf: &mut W) -> Result<usize>
        where
            W: Write,
        {
            match self {
                Ok(value) => {
                    buf.write_all(&[0])?;
                    value.encode(buf)
                }
                Err(value) => {
                    buf.write_all(&[1])?;
                    value.encode(buf)
                }
            }
        }
    }
}

mod vec {

    use std::io::{Read, Write};

    use crate::{
        Result,
        sync::{Decode, Encode},
    };

    impl<T: Encode> Encode for Vec<T> {
        fn encode<W>(&self, writer: &mut W) -> Result<usize>
        where
            W: Write,
        {
            let mut total = 0;
            total += (self.len() as u64).encode(writer)?;
            for item in self {
                total += item.encode(writer)?;
            }
            Ok(total)
        }
    }

    impl<T: Decode> Decode for Vec<T> {
        fn decode<R>(reader: &mut R) -> Result<Self>
        where
            R: Read,
        {
            let len = u64::decode(reader)? as usize;
            let mut vec = Vec::with_capacity(len);
            for _ in 0..len {
                vec.push(T::decode(reader)?);
            }
            Ok(vec)
        }
    }
}
