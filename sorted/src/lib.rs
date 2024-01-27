use proc_macro2::{TokenStream, Span};
use quote::ToTokens;
use syn::{Result, Error, parse_macro_input, Item};

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


fn sorted_impl(_args: ProcStream, input: syn::Item) -> Result<TokenStream> {
    let inner = if let Item::Enum(en) = input {
        en
    } else {
        return Err(Error::new(Span::call_site(), "expected enum or match expression"))
    };

    Ok(inner.into_token_stream())
}