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

// TODO other things

use seq::seq;

seq!(N in 1..=64 {
    pub enum B~N {}

    impl Specifier for B~N {
        const BITS: BitsType = N; 
    }
});

type BitsType = usize;

pub trait Specifier {
    const BITS: BitsType;
}



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