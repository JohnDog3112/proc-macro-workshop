use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Data, DataStruct, Result, Error, Meta, Expr, spanned::Spanned, Lit};


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

    let implementation = if let Data::Struct(val) = &input.data {
        derive_struct(&name_str, val)?
    } else {
        unimplemented!()
    };


    Ok(quote!{
        impl ::std::fmt::Debug for #name {
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
