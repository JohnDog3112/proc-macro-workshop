use proc_macro2::{TokenStream, Ident};
use syn::{Item, ItemStruct, Result, Error, LitInt};
use quote::quote;

use super::ProcStream;

pub fn bitfield_impl(args: ProcStream, input: Item) -> Result<TokenStream> {

    match input {
        Item::Struct(struc) => {
            bitfield_struct(args, struc)
        },
        _ => todo!("currently only supports structs"),
    }

    //Ok(TokenStream::new())
}

fn bitfield_struct(_args: ProcStream, struc: ItemStruct) -> Result<TokenStream> {

    let vis = struc.vis;
    let ident = struc.ident;

    let size: TokenStream = struc.fields.iter().map(|field| {
        let ty = &field.ty;
        quote!(
            + <#ty as ::bitfield::Specifier>::BITS
        )
    }).collect();

    let mut prev: TokenStream = quote!(0);

    let getters_setters: TokenStream = struc.fields.iter().map(|field| {
        let field_ident = field.ident.clone().unwrap();
        let ty = &field.ty;
        let setup = quote!(
            const PREV_BITS: usize = #prev;

            const START_BYTES: usize = PREV_BITS/8;
            const START_BITS: usize = PREV_BITS%8;

            const LAST_BITS: usize = (PREV_BITS + <#ty as ::bitfield::Specifier>::BITS);
            
            const END_BYTES: usize = LAST_BITS/8;
            const END_BITS: usize = LAST_BITS%8;
        );

        prev.extend(quote!(+ <#ty as ::bitfield::Specifier>::BITS));

        let getter = Ident::new(
            &format!("get_{field_ident}"), field_ident.span()
        );

        let setter = Ident::new(
            &format!("set_{field_ident}"), field_ident.span()
        );


        quote!(
            #vis fn #getter(&self) -> <#ty as ::bitfield::Specifier>::Ty {
                #setup

                let mut arr = [0; (<#ty as ::bitfield::Specifier>::BITS + 7)/8];

                if START_BYTES == END_BYTES {
                    const START_MASK: u8 = (0b1111_1111u8.overflowing_shr(START_BITS as u32)).0;
                    const END_MASK: u8 = (0b1111_1111u8.overflowing_shl((8 - END_BITS) as u32))
                        .0;

                    arr[0] = (self.data[START_BYTES] & START_MASK & END_MASK) >> (8 - END_BITS);

                    return ByteArray(arr).into();
                }

                // 0            1           2
                //[...._AAAA],[BBBB_BBBB],[CCCC_CCCC]
                //to
                //[...._AAAA],[BBBB_BBBB],[CCCC_CCCC]

                if END_BITS == 0 {
                    const START_MASK: u8 = 0b1111_1111u8.overflowing_shr(START_BITS as u32).0;

                    arr[0] = self.data[START_BYTES] & START_MASK;

                    #[allow(clippy::reversed_empty_ranges)]
                    arr[1..(END_BYTES - START_BYTES)].copy_from_slice(&self.data[START_BYTES+1..END_BYTES]);


                    return ByteArray(arr).into();
                }


                // 0            1           2
                //[...._ABCD],[EFFF_FFFG],[HIII_J...]
                //to
                //[...._...A],[BCDE_FFFF],[FFGH_IIIJ]

                // 0            1           2
                //[...._.BCD],[EFFF_FFFG],[HIII_J...]
                //to
                //[BCDE_FFFF],[FFGH_IIIJ]

                const OFFSET: usize = (END_BYTES - START_BYTES + 1) - (<#ty as ::bitfield::Specifier>::BITS+7)/8;

                if OFFSET == 0 {
                    //[...._ABCD] -> [...._A...] requires mask 0000_1000
                    //made with 0000_1111 & 1111_1000
                    const START_MASK: u8 = 0b1111_1111u8.overflowing_shr(START_BITS as u32).0
                        & 0b1111_1111u8.overflowing_shl((8-END_BITS) as u32).0;
                    

                    //takes [...._ABCD] -> [...._A...] -> [...._...A]
                    arr[0] = (self.data[START_BYTES] & START_MASK).overflowing_shr((8 - END_BITS) as u32).0;
                }

                //[...._ABCD] -> [...._.BCD] requires mask 0000_0111
                const A_MASK: u8 = 0b1111_1111u8.overflowing_shr(END_BITS as u32).0;

                //[EFFF_FFFG] -> [EFFF_F...] requires mask 1111_1000
                const B_MASK: u8 = 0b1111_1111u8.overflowing_shl((8 - END_BITS) as u32).0;
                
                #[allow(clippy::reversed_empty_ranges)]
                for byte in START_BYTES+1..=END_BYTES {
                    //[...._ABCD] | [EFFF_FFFG] -> [...._.BCD] | [EFFF_F...]
                    //  -> [BCD._....] | [...E_FFFF] -> [BCDE_FFFF]
                    //[...._ABCD] -> [...._.BCD] -> [BCD._....]
                    //println!("{}, {}, {}", byte-START_BYTES, byte-1, byte);
                    arr[byte-START_BYTES-OFFSET] = (self.data[byte-1] & A_MASK).overflowing_shl(END_BITS as u32).0
                        //[EFFF_FFFG] -> [EFFF_F...] -> [...E_FFFF]
                        | (self.data[byte] & B_MASK).overflowing_shr((8 - END_BITS) as u32).0;
                }


                ByteArray(arr).into()
            }

            #vis fn #setter(&mut self, mut val: <#ty as ::bitfield::Specifier>::Ty) {
                #setup

                let arr = ByteArray::<{(<#ty as ::bitfield::Specifier>::BITS+7)/8}>::from(val).0;

                //[...._ABCD] -> [...A_BCD.]
                if START_BYTES == END_BYTES || (START_BYTES+1 == END_BYTES && END_BITS == 0){
                    //0b0001_1111 & 0b1111_1110 -> 0b0001_1110

                    //0b0001_1111
                    const MASK: u8 = 0b1111_1111u8.overflowing_shr(START_BITS as u32).0
                        //0b1111_1110
                        & 0b1111_1111u8.overflowing_shl((8 - END_BITS) as u32).0;


                    const INV_MASK: u8 = MASK ^ 0b1111_1111u8;

                    //0b0001_1110 -> 0b0000_1111
                    const ARR_MASK: u8 = MASK.overflowing_shr((8 - END_BITS) as u32).0;


                    const SHIFT: usize = if END_BITS == 0 {
                        0
                    } else {
                        8 - END_BITS
                    };

                    self.data[START_BYTES] = (self.data[START_BYTES] & INV_MASK)
                        | (arr[0] & ARR_MASK) << SHIFT;

                    return;
                }


                // 0            1           2
                //[...._AAAA],[BBBB_BBBB],[CCCC_CCCC]
                //to
                //[...._AAAA],[BBBB_BBBB],[CCCC_CCCC]

                
                if END_BITS == 0 {
                    const START_MASK: u8 = 0b1111_1111u8.overflowing_shr(START_BITS as u32).0;


                    self.data[START_BYTES] = arr[0] & START_MASK;

                    #[allow(clippy::reversed_empty_ranges)]
                    self.data[START_BYTES+1..END_BYTES].copy_from_slice(&arr[1..(END_BYTES - START_BYTES)]);


                    return;
                }


                // 0            1           2
                //[...._...A],[BCDE_FFFF],[FFGH_IIIJ]
                //to
                //[...._ABCD],[EFFF_FFFG],[HIII_J...]

                // 0            1           2
                //[BCDE_FFFF],[FFGH_IIIJ]
                //to
                //[...._.BCD],[EFFF_FFFG],[HIII_J...]


                const OFFSET: usize = (END_BYTES - START_BYTES + 1) - (<#ty as ::bitfield::Specifier>::BITS+7)/8;
                
                if OFFSET == 0 {
                    //[****_***A] -> [...._...A] requires mask 0000_0001
                    const START_MASK: u8 = 0b1111_1111u8.overflowing_shr((8 - END_BITS + START_BITS) as u32).0;
                    
                    //println!("arr: 0b{:08b}", arr[0]);
                    //println!("start_mask: 0b{START_MASK:08b}");

                    //for [...._ABCD] gives 0b1111_0000
                    const DATA_MASK: u8 = 0b1111_1111u8.overflowing_shl((8 - START_BITS) as u32).0;

                    //println!("data_mask: 0b{DATA_MASK:08b}");

                    //takes [****_***A] -> [...._...A] -> [...._A...]

                    self.data[START_BYTES] = (self.data[START_BYTES] & DATA_MASK)
                        | (arr[0] & START_MASK).overflowing_shl((8 - END_BITS) as u32).0;
                                        
                    //takes [...._ABCD] -> [...._A...] -> [...._...A]
                    //arr[0] = (self.data[START_BYTES] & START_MASK).overflowing_shr((8 - END_BITS) as u32).0;
                }

                //[BCDE_FFFF] -> [BCD._....] requires mask 1110_0000
                const A_MASK: u8 = 0b1111_1111u8.overflowing_shl(END_BITS as u32).0;

                //[BCDE_FFFF] -> [...E_FFFF] requires mask 0001_1111
                const INV_A_MASK: u8 = A_MASK ^ 0b1111_1111u8;

                //[ABCD_FFFF] -> [ABCD_F...] requires mask 1111_100
                const B_MASK: u8 = 0b1111_1111u8.overflowing_shl((8 - END_BITS) as u32).0;

                const INV_B_MASK: u8 = B_MASK ^ 0b1111_1111u8;
                

                
                #[allow(clippy::reversed_empty_ranges)]
                for byte in START_BYTES+1..=END_BYTES {
                    //[...._ABCD] | [EFFF_FFFG] -> [...._.BCD] | [EFFF_F...]
                    //  -> [BCD._....] | [...E_FFFF] -> [BCDE_FFFF]
                    //[...._ABCD] -> [...._.BCD] -> [BCD._....]
                    //println!("{}, {}, {}", byte-START_BYTES, byte-1, byte);

                    //[BCDE_FFFF] -> [BCD._....] -> [...._.BCD]
                    self.data[byte-1] = (self.data[byte-1] & B_MASK)
                        | ((arr[byte-START_BYTES-OFFSET] & A_MASK) >> END_BITS);

                    //println!("p1: 0b{:08b}, 0b{:08b}", B_MASK, A_MASK);
                    
                    //[BCDE_FFFF] -> [...E_FFFF] -> [EFFF_F...]
                    self.data[byte] = (self.data[byte] & INV_B_MASK)
                        | ((arr[byte-START_BYTES-OFFSET] & INV_A_MASK) << (8 - END_BITS));

                    //println!("p2: 0b{:08b}, 0b{:08b}", A_MASK, INV_A_MASK);
                    
                }

            }
        )

    }).collect();

    let field_checks: TokenStream = struc.fields.iter().filter_map(|variant| {

        let mut num: Option<LitInt> = None;

        for attr in &variant.attrs {
            match &attr.meta {
                syn::Meta::NameValue(name_val) => {
                    match &name_val.value {
                        syn::Expr::Lit(lit) => {
                            match &lit.lit {
                                syn::Lit::Int(i_num) => {
                                    if num.is_some() {
                                        return Some(Err(Error::new_spanned(lit.clone(), "can't define more than one bit attribute!")))
                                    }
                                    num = Some(i_num.clone());
                                },
                                _ => return Some(Err(Error::new_spanned(lit.clone(), "expected integer literal"))),
                            }
                        },
                        _ => return Some(Err(Error::new_spanned(name_val.value.clone(), "expected integer literal"))),
                    }
                },
                _ => return Some(Err(Error::new_spanned(attr.clone(), "expected #[bits = <num>]")))
            }
        }

        let num = match num {
            Some(num) => num,
            None => return None,
        };

        let ty = &variant.ty ;

        let type_ident = Ident::new("tmp", num.span());

        //let type_ident = Ident::new(&format!("[(); {num}]"), num.span());
        
        Some(Ok(quote!{
            {
                let #type_ident = [(); #num];
                let _: [(); <#ty as ::bitfield::Specifier>::BITS] = #type_ident;
            }
            
        }))

    }).collect::<std::result::Result<TokenStream, Error>>()?;
    
    Ok(
        quote!(
            #vis struct #ident {
                data: [u8; (0 #size)/8]
            }

            impl #ident {
                fn new() -> Self {
                    let _ = bitfield::checks::check_mod::<[
                        ();
                        (0 #size) % 8
                    ]>();
                    #field_checks
                    Self {
                        data: [0; (0 #size)/8]
                    }
                }
                #getters_setters
            }

            impl ::bitfield::Specifier for #ident {
                const BITS: usize = (0 #size);
                type Ty = Self;
            }
        )
    )
}