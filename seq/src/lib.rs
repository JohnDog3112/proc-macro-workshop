use proc_macro2::{TokenStream, Group, TokenTree, Ident, Literal};
use syn::{Result, Token, parse_macro_input, parse::Parse, Error};


#[allow(dead_code)]
struct Seq {
    var: syn::Ident,
    lower: syn::LitInt,
    upper: syn::LitInt,
    block: Group
}
impl Parse for Seq {
    fn parse(input: syn::parse::ParseStream) -> Result<Self> {
        let var: syn::Ident = input.parse()?;
        input.parse::<Token![in]>()?;
        let lower: syn::LitInt = input.parse()?;
        input.parse::<Token![..]>()?;
        let upper: syn::LitInt = input.parse()?;

        let block = input.parse()?;

        Ok(Self {
            var,
            lower,
            upper,
            block
        })
    }
}

#[proc_macro]
pub fn seq(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    
    let seq = parse_macro_input!(input as Seq);

    seq_impl(seq)
        .unwrap_or_else(Error::into_compile_error)
        .into()
}

fn seq_impl(input: Seq) -> Result<TokenStream> {
    let Seq {
        var,
        lower,
        upper,
        block
    } = input;


    
    Ok((lower.base10_parse::<i128>()?..upper.base10_parse::<i128>()?).map(|i| {

        block.stream().clone().into_iter().map(|token| {
            replace_tokens(&var, token, i)
        }).collect::<TokenStream>()

    }).collect::<TokenStream>())

    //Ok(TokenStream::new())
}

fn replace_tokens(var: &Ident, token: TokenTree, index: i128) -> TokenTree {
    match token {
        TokenTree::Group(group) => {
            //eprintln!("Group: {}", group);
            let mut new_group = Group::new(
                group.delimiter(), 
                group.stream().into_iter().map(|tok| replace_tokens(var, tok, index)).collect()
            );
            
            new_group.set_span(group.span());
            TokenTree::Group(new_group)
        },
        TokenTree::Ident(ident) => {
            //eprintln!("Ident: {}", ident);
            if &ident == var {
                TokenTree::Literal(Literal::i128_unsuffixed(index))
            } else {
                TokenTree::Ident(ident)
            }
        }
        _ => token
    }
}
