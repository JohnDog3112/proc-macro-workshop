use std::collections::HashSet;

use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{parse_macro_input, DeriveInput, Data, DataStruct, Result, Error, Meta, Expr, spanned::Spanned, Lit, Type, PathArguments};


#[proc_macro_derive(CustomDebug, attributes(debug))]
pub fn derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);


    derive_impl(input)
        .unwrap_or_else(Error::into_compile_error)
        .into()
    
}

fn derive_impl(input: DeriveInput) -> Result<TokenStream> {
    
    let name = &input.ident;
    let name_str = name.to_string();

    let mut phantom_generics = HashSet::new();

    let implementation = if let Data::Struct(val) = &input.data {
        for field in &val.fields {
            let path = if let Type::Path(path) = &field.ty {
                path
            } else {
                continue;
            };
            
            if path.path.segments.is_empty() {
                continue;
            }

            let first_name = path.path.segments[0].ident.clone().into_token_stream().to_string();

            if first_name != "PhantomData" {
                continue;
            }

            let gens = if let PathArguments::AngleBracketed(angle) = &path.path.segments[0].arguments {
                &angle.args
            } else {
                continue;
            };

            phantom_generics.insert(gens[0].clone().into_token_stream().to_string());

        }
        derive_struct(&name_str, val)?
    } else {
        unimplemented!()
    };


    let generics = &input.generics;
    let params = &generics.params;
    let where_block = if !params.is_empty() {
        let bound_params: TokenStream = params.iter().map(|param| {
            let gen_str = param.clone().into_token_stream().to_string();

            if phantom_generics.contains(&gen_str) {
                quote!(PhantomData<#param>: ::std::fmt::Debug,)
            } else {
                quote!(#param: ::std::fmt::Debug,) 
            }
        }).collect();
        quote! {
            where
                #bound_params
        }
    } else {
        TokenStream::new()
    };

    Ok(quote!{
        impl #generics ::std::fmt::Debug for #name #generics 
        #where_block
        {
            fn fmt(&self, fmt: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                #implementation
            }
        }
    })
}


fn derive_struct(name: &str, input: &DataStruct) -> Result<TokenStream> {
    let args: TokenStream = input.fields.iter().map(|field| {
        let field_name = field.ident.clone().unwrap();
        let field_str = field_name.to_string();
        
        if !field.attrs.is_empty() {
            //eprintln!("{}", field.attrs[0].meta.clone().into_token_stream());
            let meta = if let Meta::NameValue(name) = &field.attrs[0].meta {
                Ok(name)
            } else {
                Err(Error::new_spanned(field.attrs[0].meta.clone(), "Expected `debug = \"...\"`"))
            }?;

            let literal = if let Expr::Lit(lit) = &meta.value {
                Ok(lit)
            } else {
                Err(Error::new(meta.value.span(), "Expected a String Literal! `debug = \"...\"`"))
            }?;

            let str_literal = if let Lit::Str(st) = &literal.lit {
                Ok(st)
            } else {
                Err(Error::new(literal.lit.span(), "Expected a String Literal! `debug = \"...\"`"))
            }?;


            Ok(quote!{
                .field(#field_str, &format_args!(#str_literal, &self.#field_name))
            })
        } else {
            Ok(quote! {
                .field(#field_str, &self.#field_name)
            })
        }
    }).collect::<std::result::Result<TokenStream, Error>>()?;

    Ok(quote!{
        fmt.debug_struct(#name)
            #args
            .finish()
    })
}
