pub use broto_derive::{Decode, Encode};

use futures_io::{AsyncRead, AsyncWrite};

use crate::Result;

pub trait Encode {
    fn encode<W>(&self, writer: &mut W) -> impl Future<Output = Result<usize>>
    where
        W: AsyncWrite + Unpin;
}

pub trait Decode {
    fn decode<R>(reader: &mut R) -> impl Future<Output = Result<Self>>
    where
        Self: Sized,
        R: AsyncRead + Unpin;
}

mod unit {

    use futures_io::{AsyncRead, AsyncWrite};

    use crate::{
        Result,
        r#async::{Decode, Encode},
    };

    impl Decode for () {
        async fn decode<R>(_reader: &mut R) -> Result<Self>
        where
            R: AsyncRead + Unpin,
        {
            Ok(())
        }
    }

    impl Encode for () {
        async fn encode<W>(&self, _writer: &mut W) -> Result<usize>
        where
            W: AsyncWrite + Unpin,
        {
            Ok(0)
        }
    }
}

mod tuple {

    use futures_io::{AsyncRead, AsyncWrite};

    use crate::{
        Result,
        r#async::{Decode, Encode},
    };

    impl<T> Decode for (T,)
    where
        T: Decode,
    {
        async fn decode<R>(reader: &mut R) -> Result<Self>
        where
            R: AsyncRead + Unpin,
        {
            let t = T::decode(reader).await?;
            Ok((t,))
        }
    }

    impl<T> Encode for (T,)
    where
        T: Encode,
    {
        async fn encode<W>(&self, writer: &mut W) -> Result<usize>
        where
            W: AsyncWrite + Unpin,
        {
            let (t,) = self;
            t.encode(writer).await
        }
    }

    impl<T, U> Decode for (T, U)
    where
        T: Decode,
        U: Decode,
    {
        async fn decode<R>(reader: &mut R) -> Result<Self>
        where
            R: AsyncRead + Unpin,
        {
            let t = T::decode(reader).await?;
            let u = U::decode(reader).await?;
            Ok((t, u))
        }
    }

    impl<T, U> Encode for (T, U)
    where
        T: Encode,
        U: Encode,
    {
        async fn encode<W>(&self, writer: &mut W) -> Result<usize>
        where
            W: AsyncWrite + Unpin,
        {
            let (t, u) = self;
            t.encode(writer).await?;
            u.encode(writer).await
        }
    }
}

mod numbers {
    macro_rules! decode {
        ($t:ty) => {
            impl Decode for $t {
                async fn decode<R>(reader: &mut R) -> Result<Self>
                where
                    R: AsyncRead + Unpin,
                {
                    let mut buf = [0u8; std::mem::size_of::<$t>()];
                    reader.read_exact(&mut buf).await?;
                    Ok(<$t>::from_be_bytes(buf))
                }
            }
        };
    }

    macro_rules! encode {
        ($t:ty) => {
            impl Encode for $t {
                async fn encode<W>(&self, writer: &mut W) -> Result<usize>
                where
                    W: AsyncWrite + Unpin,
                {
                    writer.write_all(&self.to_be_bytes()).await?;
                    Ok(std::mem::size_of::<$t>())
                }
            }
        };
    }

    mod unsigned {
        use futures_io::{AsyncRead, AsyncWrite};
        use futures_util::io::{AsyncReadExt, AsyncWriteExt};

        use crate::{
            Result,
            r#async::{Decode, Encode},
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

        use futures_io::{AsyncRead, AsyncWrite};
        use futures_util::io::{AsyncReadExt, AsyncWriteExt};

        use crate::{
            Result,
            r#async::{Decode, Encode},
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
        use futures_io::{AsyncRead, AsyncWrite};
        use futures_util::io::{AsyncReadExt, AsyncWriteExt};

        use crate::{
            Result,
            r#async::{Decode, Encode},
        };

        decode!(f32);
        decode!(f64);

        encode!(f32);
        encode!(f64);
    }
}

mod string {

    use futures_io::{AsyncRead, AsyncWrite};
    use futures_util::io::{AsyncReadExt, AsyncWriteExt};

    use crate::{
        Result,
        r#async::{Decode, Encode},
    };

    impl Decode for String {
        async fn decode<R>(reader: &mut R) -> Result<Self>
        where
            R: AsyncRead + Unpin,
        {
            let mut buf = [0u8; 2];
            reader.read_exact(&mut buf).await?;
            let str_len = u16::from_be_bytes(buf) as usize;
            let mut bytes = vec![0u8; str_len];
            reader.read_exact(&mut bytes).await?;
            let str = String::from_utf8_lossy(&bytes);
            Ok(str.to_string())
        }
    }

    impl Encode for String {
        async fn encode<W>(&self, buf: &mut W) -> Result<usize>
        where
            W: AsyncWrite + Unpin,
        {
            let bytes = self.as_bytes();
            let len = (bytes.len() as u16).to_be_bytes();
            buf.write_all(&len).await?;
            buf.write_all(bytes).await?;
            Ok(2 + bytes.len())
        }
    }
}

mod options {

    use futures_io::{AsyncRead, AsyncWrite};
    use futures_util::io::{AsyncReadExt, AsyncWriteExt};

    use crate::{
        Result,
        r#async::{Decode, Encode},
    };

    impl<T> Decode for Option<T>
    where
        T: Decode,
    {
        async fn decode<R>(reader: &mut R) -> Result<Self>
        where
            R: AsyncRead + Unpin,
        {
            let mut buf = [0u8; 1];
            reader.read_exact(&mut buf).await?;
            if buf[0] == 0 {
                Ok(None)
            } else {
                let value = T::decode(reader).await?;
                Ok(Some(value))
            }
        }
    }

    impl<T> Encode for Option<T>
    where
        T: Encode,
    {
        async fn encode<W>(&self, buf: &mut W) -> Result<usize>
        where
            W: AsyncWrite + Unpin,
        {
            match self {
                None => {
                    buf.write_all(&[0]).await?;
                    Ok(1)
                }
                Some(value) => {
                    buf.write_all(&[1]).await?;
                    value.encode(buf).await
                }
            }
        }
    }
}

mod result {

    use futures_io::{AsyncRead, AsyncWrite};
    use futures_util::io::{AsyncReadExt, AsyncWriteExt};

    use crate::{
        Result,
        r#async::{Decode, Encode},
    };

    impl<T, E> Decode for std::result::Result<T, E>
    where
        T: Decode,
        E: Decode,
    {
        async fn decode<R>(reader: &mut R) -> Result<Self>
        where
            R: AsyncRead + Unpin,
        {
            let mut buf = [0u8; 1];
            reader.read_exact(&mut buf).await?;
            if buf[0] == 0 {
                let value = T::decode(reader).await?;
                Ok(Ok(value))
            } else {
                let value = E::decode(reader).await?;
                Ok(Err(value))
            }
        }
    }

    impl<T, E> Encode for std::result::Result<T, E>
    where
        T: Encode,
        E: Encode,
    {
        async fn encode<W>(&self, buf: &mut W) -> Result<usize>
        where
            W: AsyncWrite + Unpin,
        {
            match self {
                Ok(value) => {
                    buf.write_all(&[0]).await?;
                    value.encode(buf).await
                }
                Err(value) => {
                    buf.write_all(&[1]).await?;
                    value.encode(buf).await
                }
            }
        }
    }
}

mod vec {

    use futures_io::{AsyncRead, AsyncWrite};

    use crate::{
        Result,
        r#async::{Decode, Encode},
    };

    impl<T: Encode> Encode for Vec<T> {
        async fn encode<W>(&self, writer: &mut W) -> Result<usize>
        where
            W: AsyncWrite + Unpin,
        {
            let mut total = 0;
            total += (self.len() as u64).encode(writer).await?;
            for item in self {
                total += item.encode(writer).await?;
            }
            Ok(total)
        }
    }

    impl<T: Decode> Decode for Vec<T> {
        async fn decode<R>(reader: &mut R) -> Result<Self>
        where
            R: AsyncRead + Unpin,
        {
            let len = u64::decode(reader).await? as usize;
            let mut vec = Vec::with_capacity(len);
            for _ in 0..len {
                vec.push(T::decode(reader).await?);
            }
            Ok(vec)
        }
    }
}
