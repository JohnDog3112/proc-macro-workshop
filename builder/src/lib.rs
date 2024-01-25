use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput, Error};

#[proc_macro_derive(Builder, attributes(builder))]
pub fn derive(input: TokenStream) -> TokenStream {
    //let _ = input;

    let input = parse_macro_input!(input as DeriveInput);

    a::derive_impl(input)
        .unwrap_or_else(Error::into_compile_error)
        .into()
    
}

mod a {
    use proc_macro2::{TokenStream, Ident, TokenTree};
    use quote::{quote, TokenStreamExt, ToTokens};
    use syn::{DeriveInput, Data, Fields, Visibility, Type, PathArguments, GenericArgument, Meta, Result, Error};


    type ParsedField = (Ident, FieldType);
    enum FieldType {
        Normal(Type),
        Optional(Type),
        Each(Type, Ident),
    }
    pub fn derive_impl(input: DeriveInput) -> Result<TokenStream> {
        //let _ = input;
        let name: Ident = input.ident;

        let data = if let Data::Struct(val) = input.data {
            val
        } else {
            unreachable!("ahhhh");
        };

        let name_builder = Ident::new(&(name.to_string() + "Builder"), name.span());
        
        let fields = if let Fields::Named(val) = data.fields {
            val
        } else {
            unreachable!("boooo");
        };

        let fields: Vec<ParsedField> = fields.named.into_iter().map(|field| {

            let ty = if let Type::Path(val) = &field.ty {
                if field.attrs.len() == 1 {
                    if let Meta::List(list) = &field.attrs[0].meta {
                        let tokens: Vec<TokenTree> = list.tokens.clone().into_iter().collect();
                        if tokens.is_empty() {
                            panic!("No arguments were provided!")
                        } else if tokens[0].to_string() != "each" {
                            //list.span()
                            return Err(Error::new_spanned(list, "expected `builder(each = \"...\")`"));
                            //unimplemented!("only each is implemented!");
                        } else if tokens.len() < 3 {
                            panic!("Too few arguments for each! each = \"<name>\"");
                        } else if tokens.len() > 3 {
                            panic!("Too many arguments for each! each = \"<name>\"");
                        }

                        if let TokenTree::Literal(lit) = &tokens[2] {
                            let str: Vec<char> = lit.to_string().chars().collect();

                            if str[0] != '"' || str[str.len()-1] != '"' {
                                panic!("Must be a normal string! \"<name>\"");
                            }

                            let field_name: String = str[1..str.len()-1].iter().collect();

                            if let PathArguments::AngleBracketed(val) = &val.path.segments[0].arguments {
                                if let GenericArgument::Type(val) = &val.args[0] {
                                    FieldType::Each(val.clone(), Ident::new(&field_name, lit.span()))
                                } else {
                                    unimplemented!("can only support static types");
                                }
                            } else {
                                unreachable!("what happened here? {}", val.path.segments[0].arguments.clone().into_token_stream());
                            }

                        } else {
                            panic!("Each can only take in a string literal!");
                        }
                    } else {
                        panic!("Invalid setup? idk");
                    }
                } else if !field.attrs.is_empty() {
                    unimplemented!("can't support more than 1 attr!");
                } else if val.path.segments[0].ident == "Option" {
                    if let PathArguments::AngleBracketed(val) = &val.path.segments[0].arguments {
                        if let GenericArgument::Type(val) = &val.args[0] {
                            FieldType::Optional(val.clone())
                        } else {
                            unimplemented!("can only support static types");
                        }
                    } else {
                        unreachable!("what happened here? {}", val.path.segments[0].arguments.clone().into_token_stream());
                    }
                } else {
                    FieldType::Normal(field.ty.clone())
                }
            } else {
                unreachable!("Invalid type!");
            };

            Ok((field.ident.clone().unwrap(), ty))
        }).collect::<std::result::Result<Vec<_>, Error>>()?;

        let new_fields: TokenStream = fields.iter().map(|field| {
            let name = &field.0;
            match &field.1 {
                FieldType::Normal(_)
                | FieldType::Optional(_) => quote!(#name: ::std::option::Option::None,),
                FieldType::Each(_, _) => quote!(#name: ::std::vec::Vec::new(),),
            }
            
        }).collect();


        

        let visibility = input.vis;
        let mut builder = quote!{
            impl #name {
                pub fn builder() -> #name_builder {
                    #name_builder {
                        #new_fields
                    }
                }
            }
        };
        builder.append_all(gen_builder_struct(&name_builder, &visibility, &fields));
        builder.append_all(gen_builder_impl(&name, &name_builder, &fields));
        Ok(builder)
    }

    fn gen_builder_struct(name: &Ident, vis: &Visibility, fields: &[ParsedField]) -> TokenStream {
        let builder_fields: TokenStream = fields.iter().map(|field| {
            let name = &field.0;
            match &field.1 {
                FieldType::Normal(ty) => quote!(#name: ::std::option::Option<#ty>,),
                FieldType::Optional(ty) => quote!(#name: ::std::option::Option<#ty>,),
                FieldType::Each(ty, _) => quote!(#name: ::std::vec::Vec<#ty>,),
            }
        }).collect();

        quote! {
            #vis struct #name {
                #builder_fields
            }
        }
    }

    fn gen_builder_impl(name: &Ident, builder_name: &Ident, fields: &[ParsedField]) -> TokenStream {
        let methods: TokenStream = fields.iter().map(|field| {
            let name = &field.0;

            if let FieldType::Each(ty, field_name) = &field.1 {
                let mut indiv = quote!{
                    pub fn #field_name(&mut self, #field_name: #ty) -> &mut Self {
                        self.#name.push(#field_name);
                        self
                    }
                };

                if field_name != &name.to_string() {
                    indiv.append_all(quote!{
                        pub fn #name(&mut self, #name: ::std::vec::Vec<#ty>) -> &mut Self {
                            self.#name = #name;
                            self
                        }
                    })
                }

                indiv
            } else {
                let ty = match &field.1 {
                    FieldType::Normal(ty) => ty.clone(),
                    FieldType::Optional(ty) => ty.clone(),
                    FieldType::Each(_,_) => unreachable!(),
                };
    
                quote!{
                    pub fn #name(&mut self, #name: #ty) -> &mut Self {
                        self.#name = ::std::option::Option::Some(#name);
                        self
                    }
                }
            }   
        }).collect();

        let build_checks: TokenStream = fields.iter().map(|field| {
            let name = &field.0;
            
            match &field.1 {
                FieldType::Normal(_) => {
                    let err = name.to_string() + " wasn't initialized!";
                    quote! {
                        #name: self.#name.clone().ok_or(#err.to_string())?,
                    }
                },
                FieldType::Optional(_)
                | FieldType::Each(_, _) => quote!(#name: self.#name.clone(),),
            }
        }).collect();

        quote! {
            impl #builder_name {
                #methods

                pub fn build(&mut self) -> ::std::result::Result<#name, String> {
                    ::std::result::Result::Ok(#name {
                        #build_checks
                    })
                } 
            }
        }
    }
}