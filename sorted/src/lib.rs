use proc_macro2::{TokenStream, Span};

use syn::{Result, Error, parse_macro_input, Item, Ident};

type ProcStream = proc_macro::TokenStream;

#[proc_macro_attribute]
pub fn sorted(args: ProcStream, mut input: ProcStream) -> ProcStream {
    let _ = args;
    let inp = input.clone();

    let item = parse_macro_input!(inp as syn::Item);

    
    let checks: ProcStream = sorted_impl(args, item)
        .unwrap_or_else(Error::into_compile_error)
        .into();

    input.extend(checks);

    input
}


fn sorted_impl(_args: ProcStream, input: syn::Item) -> Result<TokenStream> {
    let inner = if let Item::Enum(en) = input {
        en
    } else {
        return Err(Error::new(Span::call_site(), "expected enum or match expression"))
    };

    let idents: Vec<&Ident> = inner
        .variants
        .iter()
        .map(|a| &a.ident)
        .collect();

    for i in 1..idents.len() {

        if idents[i] < idents[i-1] {
            for j in 0..i {
                if idents[i] < idents[j] {
                    return Err(Error::new(
                        idents[i].span(), 
                        format!("{} should sort before {}", idents[i], idents[j])
                    ));
                }
            }
        }
    }

    //eprintln!("{:?}", inner.variants);

    Ok(TokenStream::new())
}