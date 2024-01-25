use proc_macro::TokenStream;
use syn::{Result, Token, parse_macro_input, parse::Parse};


#[allow(dead_code)]
struct Seq {
    var: syn::Ident,
    lower: syn::LitInt,
    upper: syn::LitInt,
    block: proc_macro2::TokenStream
}
impl Parse for Seq {
    fn parse(input: syn::parse::ParseStream) -> Result<Self> {
        let var: syn::Ident = input.parse()?;
        input.parse::<Token![in]>()?;
        let lower: syn::LitInt = input.parse()?;
        input.parse::<Token![..]>()?;
        let upper: syn::LitInt = input.parse()?;
        let block: proc_macro2::TokenStream = input.parse()?;
        Ok(Self {
            var,
            lower,
            upper,
            block
        })
    }
}

#[proc_macro]
pub fn seq(input: TokenStream) -> TokenStream {
    
    let Seq {
        var,
        lower,
        upper,
        block
    } = parse_macro_input!(input as Seq);

    eprintln!("{}, {}, {}, {}", var, lower, upper, block);
    
    TokenStream::new()
}
