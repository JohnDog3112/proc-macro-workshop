use std::collections::{HashSet, HashMap};

use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{parse_macro_input, DeriveInput, Data, DataStruct, Result, Error, Meta, Expr, spanned::Spanned, Lit, Type, PathArguments, Generics, Fields, GenericArgument};


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


    let (implementation, bound_generics) = if let Data::Struct(val) = &input.data {
        (
            derive_struct(&name_str, val)?, 
            get_uses_of_generics(&input.generics, &val.fields)?
        )
    } else {
        unimplemented!()
    };


    let generics = &input.generics;

    let where_block = if !bound_generics.is_empty() {
        let bound_params: TokenStream = bound_generics.into_iter().map(|param| {
            quote!{
                #param: ::std::fmt::Debug,
            }
        }).collect();
        quote! {
            where
                #bound_params
        }
    } else {
        TokenStream::new()
    };

    let oth_gen = generics.params.clone().into_iter().map(|a| {
        match a {
            syn::GenericParam::Lifetime(li) => quote!(#li,),
            syn::GenericParam::Type(ty) => {
                let ident = ty.ident;
                quote!(#ident,)
            },
            syn::GenericParam::Const(co) => quote!(#co,),
        }
    }).collect::<TokenStream>();

    Ok(quote!{
        impl #generics ::std::fmt::Debug for #name <#oth_gen>
        #where_block
        {
            fn fmt(&self, fmt: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                #implementation
            }
        }
    })
}

fn get_uses_of_generics(generics: &Generics, fields: &Fields) -> Result<Vec<Type>> {
    let mut found = HashMap::new();
    let generics: HashSet<String> = HashSet::from_iter(generics.params
        .clone()
        .into_iter()
        .filter_map(|a| {
            match a {
                syn::GenericParam::Lifetime(_) => None,
                syn::GenericParam::Type(ty) => Some(Ok(ty.ident.into_token_stream().to_string())),
                syn::GenericParam::Const(_) => Some(Err(Error::new_spanned(a, "Const's aren't supported!"))),
            }
        })
        .collect::<Result<Vec<String>>>()?
    );

    //eprintln!("{:?}", generics);

    for field in fields {
        inner_generic_uses(&generics, &field.ty, &mut found)?;
    }

    let types = found.into_values().collect::<Vec<Type>>();
    
    //eprintln!("{:?}", types.iter().map(|a| a.clone().into_token_stream().to_string()).collect::<Vec<String>>());
    Ok(types)
}

fn inner_generic_uses(generics: &HashSet<String>, ty: &Type, found: &mut HashMap<String, Type>) -> Result<()> {
    match &ty {
        Type::Path(path) => {
            for seg in &path.path.segments {
                let ident = seg.ident.clone().into_token_stream().to_string();
                
                if ident == "PhantomData" || generics.contains(&ident) {
                    found.insert(ty.clone().into_token_stream().to_string(), ty.clone());
                } else {
                    match &seg.arguments {
                        PathArguments::None => continue,
                        PathArguments::AngleBracketed(angled) => {

                            for gen in &angled.args {
                                if let Some(ty) = unwrap_generic(gen)? {
                                    inner_generic_uses(generics, ty, found)?;
                                }
                            }
                        },
                        PathArguments::Parenthesized(_) => {
                            return Err(Error::new_spanned(ty.clone(), "Paranthesized generics not supported!"));
                        },
                    }
                }
                //eprintln!("{}", seg.ident.clone().into_token_stream());
            }
        },
        Type::Tuple(tup) => {
            for elm in &tup.elems {
                inner_generic_uses(generics, elm, found)?;
            }
        },
        _a => (),
        //return Err(Error::new_spanned(ty.clone(), format!("{} isn't currently supported!",a.into_token_stream()))),
    }

    Ok(())
}

fn unwrap_generic(gen: &GenericArgument) -> Result<Option<&Type>> {
    match gen {
        syn::GenericArgument::Lifetime(_) => Ok(None),
        syn::GenericArgument::Type(ty) => Ok(Some(ty)),
        syn::GenericArgument::Const(_)
        | syn::GenericArgument::AssocType(_)
        | syn::GenericArgument::AssocConst(_)
        | syn::GenericArgument::Constraint(_) => Err(Error::new_spanned(gen.clone(), "This type isn't supported!")),
        _ => Ok(None),
    }
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


