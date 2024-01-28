use proc_macro2::{TokenStream, Ident};
use quote::quote;
use syn::{parse_macro_input, Item, Result, Error, ItemStruct};


type ProcStream = proc_macro::TokenStream;
#[proc_macro_attribute]
pub fn bitfield(args: ProcStream, input: ProcStream) -> ProcStream {
    let _ = args;
    //let  = input;

    let data = parse_macro_input!(input as Item);

    bitfield_impl(args, data)
        .unwrap_or_else(Error::into_compile_error)
        .into()
}


fn bitfield_impl(args: ProcStream, input: Item) -> Result<TokenStream> {

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
            + #ty::BITS
        )
    }).collect();

    let mut prev: TokenStream = quote!(0);

    let getters_setters: TokenStream = struc.fields.iter().map(|field| {
        let field_ident = field.ident.clone().unwrap();
        let ty = &field.ty;
        let setup = quote!(
            const prev_bits: usize = #prev;

            const start_bytes: usize = prev_bits/8;
            const start_bits: usize = prev_bits%8;

            const last_bits: usize = (prev_bits + #ty::BITS);
            
            const end_bytes: usize = last_bits/8;
            const end_bits: usize = last_bits%8;
        );

        prev.extend(quote!(+ #ty::BITS));

        let getter = Ident::new(
            &format!("get_{field_ident}"), field_ident.span()
        );

        let setter = Ident::new(
            &format!("set_{field_ident}"), field_ident.span()
        );

        quote!(
            #vis fn #getter(&self) -> u64 {
                #setup

                if start_bytes == end_bytes {
                    const start_mask: u8 = (0b1111_1111u8.overflowing_shr(start_bits as u32)).0;
                    const end_mask: u8 = (0b1111_1111u8.overflowing_shl((8-end_bits) as u32)).0;

                    return ((self.data[start_bytes] & start_mask & end_mask) >> (8 - end_bits)) as u64;
                }

                const mask: u8 = (0b1111_1111 >> start_bits);
                let mut total: u64 = (self.data[start_bytes] & mask) as u64;


                #[allow(clippy::reversed_empty_ranges)]
                for byte in start_bytes+1..end_bytes {
                    total = total.overflowing_shl(8 as u32).0 + self.data[byte] as u64;
                }

                if start_bytes != end_bytes && end_bits != 0{
                    (total << end_bits) + self.data[end_bytes].overflowing_shr((8 - end_bits) as u32).0 as u64
                    
                } else {
                    total
                }
            }

            #vis fn #setter(&mut self, mut val: u64) {
                #setup


                if start_bytes == end_bytes || (start_bytes+1 == end_bytes && end_bits == 0) {
                    const mask: u8 = 0b1111_1111u8.overflowing_shr(start_bits as u32).0
                        & 0b1111_1111u8.overflowing_shl((8-end_bits) as u32).0;

                    const rev_mask: u8 = mask ^ 0b1111_1111;

                    const val_mask: u8 = 0b1111_1111u8.overflowing_shr((8 - end_bits+start_bits) as u32).0;

                    self.data[start_bytes] = (self.data[start_bytes] & rev_mask)
                        | ((val as u8) & val_mask).overflowing_shl((8-end_bits) as u32).0;
                    
                    return;
                }


                if end_bits != 0 {
                    const end_mask_val: u8 = 0b1111_1111u8.overflowing_shr((8-end_bits) as u32).0;
                    const end_mask_data: u8 = 0b1111_1111u8.overflowing_shr(end_bits as u32).0;


                    self.data[end_bytes] = (((val as u8) & end_mask_val) << (8-end_bits))
                        | (self.data[end_bytes] & end_mask_data);
                    
                    val >>= end_bits;
                }

                #[allow(clippy::reversed_empty_ranges)]
                for byte in (start_bytes+1..end_bytes).rev() {
                    self.data[byte] = (val & 0b1111_1111) as u8;
                    val >>= 8;
                }

                const start_mask_val: u8 = 0b1111_1111u8.overflowing_shr(start_bits as u32).0;
                const start_mask_data: u8 = start_mask_val ^ 0b1111_1111;

                self.data[start_bytes] = ((val as u8) & start_mask_val) | (self.data[start_bytes] & start_mask_data);
                
            }
        )

    }).collect();
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
                    Self {
                        data: [0; (0 #size)/8]
                    }
                }
                #getters_setters
            }

            impl ::bitfield::Specifier for #ident {
                const BITS: usize = if (0 #size)%8 == 0 {
                    0 #size
                } else {
                    panic!("invalid!")
                };
            }
        )
    )
}