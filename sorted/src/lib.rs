use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::{Result, Error, parse_macro_input};

type ProcStream = proc_macro::TokenStream;

#[proc_macro_attribute]
pub fn sorted(args: ProcStream, input: ProcStream) -> ProcStream {
    let _ = args;
    let inp = input;

    let item = parse_macro_input!(inp as syn::Item);

    sorted_impl(args, item)
        .unwrap_or_else(Error::into_compile_error)
        .into()
}


fn sorted_impl(args: ProcStream, input: syn::Item) -> Result<TokenStream> {
    eprintln!("{:?}", input);

    Ok(input.into_token_stream())
}