use proc_macro2::{TokenStream, Span};

use quote::ToTokens;
use syn::{Result, Error, parse_macro_input, Item, Ident, ItemFn, visit_mut::{VisitMut, self}, Attribute};

type ProcStream = proc_macro::TokenStream;

#[proc_macro_attribute]
pub fn sorted(args: ProcStream, mut input: ProcStream) -> ProcStream {
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

    let idents: Vec<(&Ident, &dyn ToTokens)> = inner
        .variants
        .iter()
        .map(|a| (&a.ident, &a.ident as &dyn ToTokens))
        .collect();


    find_out_of_order(idents)?;

    //eprintln!("{:?}", inner.variants);

    Ok(TokenStream::new())
}

fn find_out_of_order(idents: Vec<(&Ident, &dyn ToTokens)>) -> Result<()> {


    
    for i in 1..idents.len() {

        if idents[i].0 < idents[i-1].0 {
            for j in 0..i {
                if idents[i].0 < idents[j].0 {
                    let i_a: String = idents[i].1
                        .to_token_stream()
                        .to_string()
                        .chars()
                        .filter(|a| *a != ' ')
                        .collect();
                    let i_b: String = idents[j].1
                        .to_token_stream()
                        .to_string()
                        .chars()
                        .filter(|a| *a != ' ')
                        .collect();
                    
                    return Err(Error::new_spanned(
                        idents[i].1.to_token_stream(),
                        format!("{} should sort before {}", i_a, i_b)
                    ));
                    /*return Err(Error::new(
                        idents[i].1, 
                        format!("{} should sort before {}", idents[i].0, idents[j].0)
                    ));*/
                }
            }
        }
    }
    
    Ok(())
}

struct MatchCheck(Vec<Error>);

impl VisitMut for MatchCheck {
    fn visit_expr_match_mut(&mut self, mat: &mut syn::ExprMatch) {

        let mut tmp = Vec::new();

        std::mem::swap(&mut tmp, &mut mat.attrs);

        let tmp: std::result::Result<Vec<Attribute>, Error> = tmp.into_iter().filter_map(|attr| {
            eprintln!("{}", attr.meta.to_token_stream());

            let path = if let syn::Meta::Path(path) = &attr.meta {
                path
            } else {
                return Some(Ok(attr))
            };

            let ident = if let Some(ident) = path.get_ident() {
                ident
            } else {
                return Some(Ok(attr))
            };

            if ident != "sorted" {
                return Some(Ok(attr));
            }


            let idents = mat.arms.iter().map(|arm| {

                match &arm.pat {
                    syn::Pat::TupleStruct(tuple_struct) => {
                        //eprintln!("{:?}", );
                        (
                            &tuple_struct.path.segments.last().unwrap().ident, 
                            &tuple_struct.path as &dyn ToTokens
                        )
                    },
                    syn::Pat::Ident(ident) => {
                        (&ident.ident, &ident.ident as &dyn ToTokens)
                    },
                    syn::Pat::Struct(struc) => {
                        (
                            &struc.path.segments.last().unwrap().ident, 
                            &struc.path as &dyn ToTokens
                        )
                    },
                    _ => todo!(),
                }
            }).collect();
            if let Err(a) = find_out_of_order(idents) {
                Some(Err(a))
            } else {
                None
            }
        }).collect();


        match tmp {
            Ok(mut attrs) => {
                std::mem::swap(&mut attrs, &mut mat.attrs);
                visit_mut::visit_expr_match_mut(self, mat);
            },
            Err(err) => {
                self.0.push(err)
            },
        }
    }
}
#[proc_macro_attribute]
pub fn check(args: ProcStream, input: ProcStream) -> ProcStream {

    let item = parse_macro_input!(input as ItemFn);


    check_impl(args, item)
        //.unwrap_or_else(Error::into_compile_error)
        .into()
}

fn check_impl(_args: ProcStream, input: ItemFn) -> TokenStream {

    /*for token in &input.block.stmts {
        eprintln!("{}", token.to_token_stream());
        eprintln!("\n")
    }*/

    let mut input = input;

    let mut match_check = MatchCheck(Vec::new());

    match_check.visit_block_mut(&mut input.block);

    if match_check.0.is_empty() {
        input.into_token_stream()
    } else {
        let mut stream = input.into_token_stream();
        let err_stream = match_check.0.into_iter().reduce(|mut acc, e| {
            acc.combine(e);
            acc
        }).unwrap().into_compile_error();

        stream.extend(err_stream);

        stream
    }
    //Ok(input.into_token_stream())
}
