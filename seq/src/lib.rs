use proc_macro2::{TokenStream, Group, TokenTree, Ident, Literal};
use syn::{Result, Token, parse_macro_input, parse::Parse, Error};


#[allow(dead_code)]
struct Seq {
    var: syn::Ident,
    lower: syn::LitInt,
    upper: syn::LitInt,
    inclusive: bool,
    block: Group
}
impl Parse for Seq {
    fn parse(input: syn::parse::ParseStream) -> Result<Self> {
        let var: syn::Ident = input.parse()?;
        input.parse::<Token![in]>()?;
        let lower: syn::LitInt = input.parse()?;
        input.parse::<Token![..]>()?;

        let inclusive = if input.peek(Token![=]) {
            input.parse::<Token![=]>()?;
            true
        } else {
            false
        };

        let upper: syn::LitInt = input.parse()?;


        let block = input.parse()?;

        Ok(Self {
            var,
            lower,
            upper,
            inclusive,
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
        inclusive,
        block,
    } = input;


    
    let lower = lower.base10_parse::<i128>()?;
    let upper = upper.base10_parse::<i128>()? + if inclusive {1} else {0};


    let (out_stream, groups_found) = find_replace_groups(lower, upper, &var, block.stream());

    if groups_found > 0 {
        Ok(out_stream)
    } else {
        Ok((lower..upper).map(|i| {

            /*block.stream().clone().into_iter().map(|token| {
                replace_tokens(&var, token, i)
            }).collect::<TokenStream>()*/
    
            replace_tokens(&var, block.stream(), i)
    
        }).collect::<TokenStream>())
    }

    //Ok(TokenStream::new())
}

fn find_replace_groups(lower: i128, upper: i128, var: &Ident, tokens: TokenStream) -> (TokenStream, usize) {
    //eprintln!("{}", tokens);
    let mut tokens: Vec<TokenTree> = tokens.into_iter().collect();

    let mut groups_found = 0;

    let mut index = 0;

     

    while index < tokens.len() {
        //eprintln!("iter: {}", tokens[index]);

        //check for # in #(<repeat>)*
        index += 1;
        match &tokens[index-1] {
            TokenTree::Punct(punct) => {
                if punct.as_char() != '#' {
                    continue;
                }
            },
            TokenTree::Group(group) => {
                let (stream, found) = find_replace_groups(lower, upper, var, group.stream());
                groups_found += found;

                let mut new_group = TokenTree::Group(
                    Group::new(
                        group.delimiter(),
                        stream
                    )
                );
                new_group.set_span(group.span());

                tokens[index-1] = new_group;
                continue;

            },
            _ => continue,
        }

        //checks that there's enough left in token stream for a repeat section
        if index+1 >= tokens.len() {
            continue;
        }

        //checks for * in #(<repeat>)*
        if let TokenTree::Punct(punct) = &tokens[index+1] {
            if punct.as_char() != '*' {
                continue;
            }
        } else {
            continue;
        }

        //checking inner group <repeat> in #(<repeat>)*
        if let TokenTree::Group(group) = &tokens[index] {
            //ensuring that the group is in parenthesis
            match group.delimiter() {
                proc_macro2::Delimiter::Parenthesis => (),
                _ => continue
            }

            let new_tokens = (lower..upper).map(|i| {
                replace_tokens(var, group.stream(), i)
            }).collect::<TokenStream>();

            let bef_len = tokens.len();

            let tmp = tokens.into_iter();
            
            tokens = Vec::from_iter(
                tmp.clone().take(index-1)
                    .chain(new_tokens)
                    .chain(tmp.skip(index+2))
            );

            index += tokens.len()-bef_len+2;

            groups_found += 1;
        } else {
            continue;
        }
        
    }

    (tokens.into_iter().collect(), groups_found)
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
