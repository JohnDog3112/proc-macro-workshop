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

        /*block.stream().clone().into_iter().map(|token| {
            replace_tokens(&var, token, i)
        }).collect::<TokenStream>()*/

        replace_tokens(&var, block.stream(), i)

    }).collect::<TokenStream>())

    //Ok(TokenStream::new())
}

fn replace_tokens(var: &Ident, tokens: TokenStream, index: i128) -> TokenStream {

    enum CheckState {
        Nothing,
        Var,
        Tilda
    }

    let mut parsed_tokens: Vec<TokenTree> = vec![];

    let mut check_state = CheckState::Nothing;

    for token in tokens {

        if let CheckState::Tilda = check_state {
            check_state = CheckState::Nothing;

            let prev = parsed_tokens.pop().unwrap();

            parsed_tokens.push(
                TokenTree::Ident(
                    Ident::new(
                        &format!("{}{}", prev, token),
                        prev.span()
                    )
                )
            );
            continue;
        }
        match token {
            TokenTree::Group(group) => {
                //eprintln!("Group: {}", group);

                let mut new_group = Group::new(
                    group.delimiter(),
                    replace_tokens(var, group.stream(), index)
                );
    
                new_group.set_span(group.span());
                parsed_tokens.push(TokenTree::Group(new_group));
            },
            TokenTree::Ident(ident) => {
                //eprintln!("Ident: {}", ident);
                let val = if &ident == var {
                    check_state = CheckState::Var;

                    let literal = TokenTree::Literal(Literal::i128_unsuffixed(index));

                    if let Some(prev) = parsed_tokens.last() {
                        match prev {
                            TokenTree::Punct(punc) => {
                                if punc.as_char() == '~' {
                                    parsed_tokens.pop();
                                    let prev = parsed_tokens.pop().unwrap();

                                    TokenTree::Ident(Ident::new(&format!("{}{}", prev, index), prev.span()))
                                } else {
                                    literal
                                }
                            },
                            _ => literal
                        }
                    } else {
                        literal
                    }
                    
                } else {
                    TokenTree::Ident(ident)
                };

                parsed_tokens.push(val);
            },
            TokenTree::Punct(punct) => {
                if punct.as_char() == '~' {
                    if let CheckState::Var = check_state {
                        check_state = CheckState::Tilda;
                    } else {
                        parsed_tokens.push(TokenTree::Punct(punct));
                    }
                } else {
                    parsed_tokens.push(TokenTree::Punct(punct));
                }
            }
            _ => parsed_tokens.push(token)
        }

    }

    parsed_tokens.into_iter().collect()

}
