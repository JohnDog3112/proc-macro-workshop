use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Data, DataStruct};


#[proc_macro_derive(CustomDebug)]
pub fn derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {

    let input = parse_macro_input!(input as DeriveInput);

    
    let name = &input.ident;
    let name_str = name.to_string();

    let implementation = if let Data::Struct(val) = &input.data {
        derive_struct(&name_str, val)
    } else {
        unimplemented!()
    };


    quote!{
        impl ::std::fmt::Debug for #name {
            fn fmt(&self, fmt: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                #implementation
            }
        }
    }.into()
}


fn derive_struct(name: &str, input: &DataStruct) -> TokenStream {
    let args: TokenStream = input.fields.iter().map(|field| {
        let field_name = field.ident.clone().unwrap();
        let field_str = field_name.to_string();
        
        quote! {
            .field(#field_str, &self.#field_name)
        }
    }).collect();

    quote!{
        fmt.debug_struct(#name)
            #args
            .finish()
    }
}
