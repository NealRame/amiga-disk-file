use std::io::{
    Error,
    Read,
};

use paste::paste;

macro_rules! generate_integer_reader {
    ($($t:ty),*) => {
        pub trait IntegerReader {
            paste! {
                $(
                    fn [<try_read_ $t>](
                        reader: &mut impl Read,
                    ) -> Result<$t, Error>;
                )*
            }
        }

        pub struct LSB ();
        pub struct MSB ();

        paste! {
            impl IntegerReader for LSB {
                $(
                    fn [<try_read_ $t>](
                        reader: &mut impl Read,
                    ) -> Result<$t, Error> {
                        let mut buf = [0; std::mem::size_of::<$t>()];
                        reader.read(&mut buf)?;
                        Ok($t::from_le_bytes(buf))
                    }
                )*
            }

            impl IntegerReader for MSB {
                $(
                    fn [<try_read_ $t>](
                        reader: &mut impl Read,
                    ) -> Result<$t, Error> {
                        let mut buf = [0; std::mem::size_of::<$t>()];
                        reader.read(&mut buf)?;
                        Ok($t::from_be_bytes(buf))
                    }
                )*
            }
        }

        pub trait TryReadFrom: Sized {
            fn try_read_from<Ord: IntegerReader>(
                reader: &mut impl Read
            ) -> Result<Self, Error>;
        }

        paste! {
            $(
                impl TryReadFrom for $t {
                    fn try_read_from<Reader: IntegerReader>(
                        reader: &mut impl Read
                    ) -> Result<Self, Error> {
                        Reader::[<try_read_ $t>](reader)
                    }
                }
            )*
        }
    };
}

generate_integer_reader!(
    i8, u8,
    i16, u16,
    i32, u32,
    i64, u64,
    i128, u128
);
