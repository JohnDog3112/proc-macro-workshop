// Crates that have the "proc-macro" crate type are only allowed to export
// procedural macros. So we cannot have one crate that defines procedural macros
// alongside other types of public APIs like traits and structs.
//
// For this project we are going to need a #[bitfield] macro but also a trait
// and some structs. We solve this by defining the trait and structs in this
// crate, defining the attribute macro in a separate bitfield-impl crate, and
// then re-exporting the macro from this crate so that users only have one crate
// that they need to import.
//
// From the perspective of a user of this crate, they get all the necessary APIs
// (macro, trait, struct) through the one bitfield crate.
pub use bitfield_impl::bitfield;
pub use bitfield_impl::BitfieldSpecifier;

// TODO other things

pub const fn max_usize(a: usize, b: usize) -> usize {
    if a < b {
        b
    } else {
        a
    }
}

use seq::seq;

seq!(N in 1..=8 {
    pub enum B~N {}

    impl Specifier for B~N {
        const BITS: BitsType = N;
        type Ty = u8;
    }
});

seq!(N in 9..=16 {
    pub enum B~N {}

    impl Specifier for B~N {
        const BITS: BitsType = N;
        type Ty = u16;
    }
});

seq!(N in 17..=32 {
    pub enum B~N {}

    impl Specifier for B~N {
        const BITS: BitsType = N;
        type Ty = u32;
    }
});

seq!(N in 33..=64 {
    pub enum B~N {}

    impl Specifier for B~N {
        const BITS: BitsType = N;
        type Ty = u64;
    }
});

impl Specifier for bool {
    const BITS: BitsType = 1;
    type Ty = bool;
}

type BitsType = usize;



pub trait Specifier {
    const BITS: BitsType;
    type Ty;
}

fn primitive_into_arr<T, const N: usize>(mut val: T) -> Result<ByteArray<N>, core::num::TryFromIntError> 
where
    T: std::ops::Add,
    T: TryInto<u8, Error = core::num::TryFromIntError>,
    T: From<u8>,
    T: std::ops::BitAnd<Output = T>,
    T: std::ops::ShrAssign<u8>,
    T: Copy,
{
    let mut arr = [0u8; N];

    for i in (0..N).rev() {
        arr[i] = (
            val 
            & T::from(0b1111_1111)
        ).try_into()?;

        val >>= 8;
    }

    Ok(ByteArray(arr))
}

impl From<u8> for ByteArray<1> {
    fn from(value: u8) -> Self {
        ByteArray([value])
    }
}

impl From<u16> for ByteArray<2> {
    fn from(value: u16) -> Self {
        primitive_into_arr(value).unwrap()
    }
}

seq!(N in 3..=4 {
    impl From<u32> for ByteArray<N> {
        fn from(value: u32) -> Self {
            primitive_into_arr(value).unwrap()
        }
    }
});


seq!(N in 0..=8 {
    impl From<u64> for ByteArray<N> {
        fn from(value: u64) -> Self {
            primitive_into_arr(value).unwrap()
        }
    }
});

seq!(N in 9..=16 {
    impl From<u128> for ByteArray<N> {
        fn from(value: u128) -> Self {
            primitive_into_arr(value).unwrap()
        }
    }
});

impl From<bool> for ByteArray<1> {
    fn from(value: bool) -> Self {
        ByteArray(if value {
            [1;1]
        } else {
            [0;1]
        })
    }
}


impl From<ByteArray<1>> for u8 {
    fn from(value: ByteArray<1>) -> Self {
        value.0[0]
    }
}

impl From<ByteArray<2>> for u16 {
    fn from(val: ByteArray<2>) -> Self {
        ((val.0[0] as u16) << 8) + val.0[1] as u16
    }
}

macro_rules! expand_inner {
    ($val:ident, $n:literal) => {
        seq!(G in 0..$n { 
            0
            #(
            + (($val.0[G] as Self) << (8*($n-G-1)))
            )*
         })
    }
}

seq!(N in 3..=4 {
    impl From<ByteArray<N>> for u32 {
        fn from(val: ByteArray<N>) -> Self {
            expand_inner!(val, N)
        }
    }
});



impl From<ByteArray<1>> for u64 {
    fn from(val: ByteArray<1>) -> Self {
        val.0[0] as Self
    }
}
seq!(N in 2..=8 {
    impl From<ByteArray<N>> for u64 {
        fn from(val: ByteArray<N>) -> Self {
            expand_inner!(val, N)
        }
    }
});

seq!(N in 9..=16 {
    impl From<ByteArray<N>> for u128 {
        fn from(val: ByteArray<N>) -> Self {
            expand_inner!(val, N)
        }
    }
});

impl From<ByteArray<1>> for bool {
    fn from(val: ByteArray<1>) -> Self {
        val.0[0] != 0
    }
}

#[derive(Debug)]
pub struct ByteArray<const N: usize>(pub [u8; N]);


pub mod checks {

    pub fn check_mod<T: CheckMod8>() -> u32 
    where 
        T::Num: MultipleOf8,
    {
        check_mod_8::<T::Num>();
        0
    }

    fn check_mod_8<T: MultipleOf8>() {
        
    }

    pub trait MultipleOf8 {}
    pub trait CheckMod8 {
        type Num;
    }



    pub struct ZeroMod8 {}
    impl CheckMod8 for [(); 0] {
        type Num = ZeroMod8;
    }
    impl MultipleOf8 for ZeroMod8 {}

    pub struct OneMod8 {}
    impl CheckMod8 for [(); 1] {
        type Num = OneMod8;
    }
    pub struct TwoMod8 {}
    impl CheckMod8 for [(); 2] {
        type Num = TwoMod8;
    }
    pub struct ThreeMod8 {}
    impl CheckMod8 for [(); 3] {
        type Num = ThreeMod8;
    }
    pub struct FourMod8 {}
    impl CheckMod8 for [(); 4] {
        type Num = FourMod8;
    }
    pub struct FiveMod8 {}
    impl CheckMod8 for [(); 5] {
        type Num = FiveMod8;
    }
    pub struct SixMod8 {}
    impl CheckMod8 for [(); 6] {
        type Num = SixMod8;
    }
    pub struct SevenMod8 {}
    impl CheckMod8 for [(); 7] {
        type Num = SevenMod8;
    }


}