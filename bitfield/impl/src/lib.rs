use proc_macro2::TokenStream;
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

    let size: TokenStream = struc.fields.into_iter().map(|field| {
        let ty = field.ty;
        quote!(
            + #ty::BITS
        )
    }).collect();


    Ok(
        quote!(
            #vis struct #ident {
                data: [u8; (0 #size)/8]
            }
        )
    )
}