use proc_macro2::{TokenStream, Span};
use quote::quote;
use syn::{parse_macro_input, Item, Result, Error, DeriveInput, DataEnum};


type ProcStream = proc_macro::TokenStream;
mod bitfield;
#[proc_macro_attribute]
pub fn bitfield(args: ProcStream, input: ProcStream) -> ProcStream {
    let _ = args;
    //let  = input;

    let data = parse_macro_input!(input as Item);

    bitfield::bitfield_impl(args, data)
        .unwrap_or_else(Error::into_compile_error)
        .into()
}






#[proc_macro_derive(BitfieldSpecifier)]
pub fn bitfield_specifier(input: ProcStream) -> ProcStream {

    let data = parse_macro_input!(input as DeriveInput);


    bitfield_specifier_input(data)
        .unwrap_or_else(Error::into_compile_error)
        .into()
}

fn bitfield_specifier_input(inp: DeriveInput) -> Result<TokenStream> {
    match &inp.data {
        syn::Data::Enum(enu) => bitfield_specifier_enum(&inp, enu),
        _ => todo!(),
    }
}

fn bitfield_specifier_enum(inp: &DeriveInput, enu: &DataEnum) -> Result<TokenStream> {
    let ident = &inp.ident;

    //eprintln!("hi there!, {:?}", enu.variants);
    let variants = enu.variants.iter().map(|variant| {
        match &variant.fields {
            syn::Fields::Unit => (),
            _ => return Err(Error::new_spanned(variant.clone(), "must be unit type!"))
        };



        Ok(&variant.ident)
    }).collect::<std::result::Result<Vec<_>, Error>>()?;

    let mag = variants.len().ilog2() as usize;

    if 2usize.pow(mag as u32) != variants.len() {
        return Err(Error::new(Span::call_site(),"BitfieldSpecifier expected a number of variants which is a power of 2"));
    }

    let bit_size: TokenStream = variants.iter().fold(quote!(0), |acc, e| {
        quote!{
            ::bitfield::max_usize(#acc, #ident::#e as usize)
        }
    });

    let from_sets: TokenStream = variants.iter().map(|variant| {
        quote!{
            a if a == #ident::#variant as u64 => #ident::#variant,
        }
    }).collect();

    

    //eprintln!("magnitude: {}", mag);
    Ok(quote!{
        impl ::bitfield::Specifier for #ident {
            const BITS: usize = #mag;
            type Ty = #ident;
        }

        impl #ident {
            pub fn new() -> Self{
                let _ = ::bitfield::checks::check_in_range::<[(); if #bit_size as usize == 2usize.pow(Self::BITS as u32)-1 {1} else {0}]>();

                Self::try_from(0u64).unwrap()
            }
        }

        impl TryFrom<u64> for #ident {
            type Error = ();
            fn try_from(val: u64) -> Result<Self, Self::Error> {
                Ok(match val {
                    #from_sets
                    _ => return Err(())
                })
            }
        }

        impl From<#ident> for u64 {
            fn from(val: #ident) -> Self {
                val as u64
            }
        }

        impl From<ByteArray<{(Self::BITS+7)/8}>> for #ident {
            fn from(val: ByteArray<{(Self::BITS+7)/8}>) -> Self {
                u64::from(val).try_into().unwrap()
            }
        }

        impl From<#ident> for ByteArray<{(#mag +1+7)/8usize}> {
            fn from(val: #ident) -> Self {
                u64::from(val).into()
            }
        }
    })
}