use convert_case::{Case, Casing};
use proc_macro2::TokenStream;
use quote::quote;
use syn::{spanned::Spanned, *};

/// Defines style formating
const INSTRUCTION_ARGUMENTS_CASE: Case = Case::Snake;
const PROMETHEUS_CASE: Case = Case::Snake;

/// This func parseed struct fields and returns lines which push `PathTree` struct for some field into `fields_vec` vector.
///
/// `fields_vec` vector must be defined before calling `TokenStream` which returns this func.
fn parse_struct_fields(fields: &syn::Fields, type_name: &syn::Ident) -> TokenStream {
    let mut from_code = Vec::new();

    match fields {
        syn::Fields::Named(syn::FieldsNamed {
            named: named_fields,
            ..
        }) => {
            named_fields
                .into_iter()
                .for_each(|named_field| {
                    let field_ident = named_field.ident.as_ref().unwrap();

                    from_code.push(quote!{
                        fields_vec.push((stringify!(#field_ident).to_string(), Box::new(other_val.#field_ident.into())));
                    });
                });
        }
        syn::Fields::Unnamed(syn::FieldsUnnamed {
            unnamed: unnamed_fields,
            ..
        }) => {
            unnamed_fields.into_iter().enumerate().for_each(|(i, unnamed_field)| {
                let index = syn::Index{index: i as u32, span: unnamed_field.span()};

                from_code.push(quote!{
                    fields_vec.push((stringify!(#index).to_string(), Box::new(other_val.#index.into())));
                });
            });
        }
        syn::Fields::Unit => {
            let type_name_formatted = type_name.to_string().to_case(INSTRUCTION_ARGUMENTS_CASE);
            from_code.push(quote! {
                fields_vec.push((#type_name_formatted.to_string(), Box::new(PathTree::None)));
            });
        }
    }

    quote! {#(#from_code)*}
}

/// Parse Enum Variants and returns `match self` expression
/// that returns string containing variant name and pushes `PathTree` struct for all fields, contained in
/// this variant, into `fields_vec` vector.
///
/// `fields_vec` vector must be defined before calling `TokenStream` which returns this func.
fn parse_enum_variants(
    variants: &syn::punctuated::Punctuated<syn::Variant, syn::token::Comma>,
    enum_ident: &syn::Ident,
) -> TokenStream {
    let mut from_code = Vec::new();

    // Generating match arm
    variants.into_iter().for_each(|variant| {
        let mut math_arm_inner_code = Vec::new();
        let variant_ident = &variant.ident;
        let variant_name_formatted = variant_ident.to_string().to_case(INSTRUCTION_ARGUMENTS_CASE);
        let variant_fields = &variant.fields;

        // This var below represents list of idents, passed into match arm.
        // # Example
        //
        // SomeEnum::Unit => {...}
        // In this case args var would be quote!{ }

        // SomeEnum::TupleVariant2(arg0, arg1, arg2) => {...}
        // In this case args var would be quote!{ (arg0, arg1, arg2) }
        //
        // SomeEnum::StructLikeVariant2{field_in_struct_like_variant1, field2, some_other_field} => {...}
        // In this case args would be quote!{ {field_in_struct_like_variant1, field2, some_other_field} }
        let args = match variant_fields {
            syn::Fields::Named(syn::FieldsNamed { named: named_fields, .. }) => {
                let args_vec: Vec<&syn::Ident> = named_fields
                .into_iter()
                .map(|named_field| {
                    let named_field_ident = named_field.ident.as_ref().unwrap();

                    math_arm_inner_code.push(quote!{
                        fields_vec.push((stringify!(#named_field_ident).to_string(), Box::new(#named_field_ident.into())));
                    });

                   named_field_ident
                }).collect();
                if args_vec.is_empty() { quote!{} } else { quote!{{#(#args_vec, )*}} }
            }
            syn::Fields::Unnamed(syn::FieldsUnnamed {
                unnamed: unnamed_fields, ..
            }) => {
                let args_vec: Vec<syn::Ident> = unnamed_fields.into_iter().enumerate().map(|(i, unnamed_field)| {

                    let index = syn::Index{index: i as u32, span: unnamed_field.span()};
                    let arg = syn::Ident::new(format!("arg{}", i).as_str(), unnamed_field.span());

                    math_arm_inner_code.push(quote!{
                        fields_vec.push((stringify!(#index).to_string(), Box::new(#arg.into())));
                    });

                    arg
                }).collect();

                if args_vec.is_empty() { quote!{} } else { quote!{(#(#args_vec, )*)} }
            }
            syn::Fields::Unit => {
                quote!{}
            }
        };


        from_code.push(quote! {
                #enum_ident::#variant_ident #args => {
                    #(#math_arm_inner_code)*
                    #variant_name_formatted
                },
        });
    });

    quote! {
        match other_val {
            #(#from_code)*
        }
    }
}

/// Generates impl From<T> for PathTree for additional types such as array and tuple.
/// You can pass `Array(..)` or 'Tuple(..)' attribute in macros with inner parameters, which represents length of array of tuple.
/// E.G: #[implement_path_tree(Array(1, 2), Tuple(2))] - it will generate 'impl From<T> for PathTree' code for array of length 1 and 2 and tuple of length 2,
/// regardless of types.
///
/// Note: macro `implement_path_tree` is just a wrapper of this func.
fn get_additional_impls(attr: &syn::AttributeArgs) -> TokenStream {
    let mut additional_impls = Vec::new();
    attr.iter().for_each(|meta| {
        match meta {
            syn::NestedMeta::Meta(meta) => {
                if let syn::Meta::List(syn::MetaList{path, nested, ..}) = meta {
                    if path.is_ident("Tuple") {
                        nested.iter().for_each(|nested_meta| {
                            match nested_meta {
                                syn::NestedMeta::Lit(syn::Lit::Int(lit)) => {
                                    let mut inner_code = Vec::new();
                                    let mut templates = Vec::new();
                                    let length = lit.base10_parse::<usize>().unwrap();

                                    for i in 0..length {
                                        let index = syn::Index::from(i);
                                        inner_code.push(quote! {
                                            tuple_fields.push((stringify!(#index).to_string(), Box::new(other_val.#index.into())));
                                        });
                                        let ident = syn::Ident::new(format!("T{}", i).as_str(), path.span());
                                        templates.push(ident);
                                    }

                                    inner_code.push(quote!{
                                        fields_vec.push(("".to_string(), Box::new(PathTree::Path(tuple_fields))));
                                    });

                                    additional_impls.push(quote! {
                                        impl<#(#templates, )*> From<(#(#templates, )*)> for PathTree
                                        where #(#templates: Into<PathTree> + Clone, )*
                                        {
                                            fn from(other_val: (#(#templates, )*)) -> Self {
                                                let mut tuple_fields = Vec::new();
                                                let mut fields_vec = Vec::new();
                                                #(#inner_code)*

                                                PathTree::Path(fields_vec)
                                            }
                                        }
                                    });
                                }
                                _ => {
                                    panic!("Tuple attribute can contain only length field");
                                }
                            }
                        });
                    } else if path.is_ident("Array") {
                        nested.iter().for_each(|nested_meta| {
                            let length  = if let syn::NestedMeta::Lit(syn::Lit::Int(lit)) = &nested_meta {
                                lit.base10_parse::<usize>().unwrap()
                            } else {
                                panic!("Array attributes must be an usize value");
                            };


                            additional_impls.push(quote!{
                                impl<T> From<[T; #length]> for PathTree
                                where T: Into<PathTree> + Clone,
                                {
                                    fn from(slice: [T; #length]) -> Self {
                                        let mut path_vec = Vec::new();
                                        slice.into_iter().enumerate().for_each(|(i, val)| {
                                            path_vec.push((i.to_string(), Box::new(val.clone().into())));
                                        });

                                        Self::Path(path_vec)
                                    }
                                }
                            });
                        });
                    }
                } else {
                    unimplemented!("Only list is supported as attribute");
                }
            }
            _ => {
                unimplemented!{"Unsupported attribute argument type"};
            }
        }
    });

    quote!(#(#additional_impls)*)
}

/// Generates impl From<T> for PathTree for additional types such as array and tuple.
/// You can pass `Array(..)` or 'Tuple(..)' attribute in macros with inner parameters, which represents length of array or tuple.
/// E.G: #[implement_path_tree(Array(1, 2), Tuple(2))] - it will generate 'impl From<T> for PathTree' code for array of length 1 and 2 and tuple of length 2,
/// regardless of types.
///
/// Note: This macro is just a wrapper of `get_additional_impls` func
#[proc_macro_attribute]
pub fn implement_path_tree(
    attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let item = parse_macro_input!(item as syn::Item);
    let attr = parse_macro_input!(attr as syn::AttributeArgs);

    let additional_impls = get_additional_impls(&attr);

    quote! {
        #item
        #additional_impls
    }
    .into()
}

/// Macros produces implementation for trait From<T> for PathTree struct and method
/// `get_arguments(..)`, that returns Vec<InstructionArgument>
///
/// Attributes:
/// * InstrRoot:  It indicates, that particular enum is "root" and won't generate it's
/// variant field name (instruction name) in `arg_path` field.

#[proc_macro_attribute]
pub fn instr_args_parse(
    attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let item = parse_macro_input!(item as syn::Item);
    let attr = parse_macro_input!(attr as syn::AttributeArgs);

    let trait_impl = match item {
        syn::Item::Struct(strct) => {
            let name = &strct.ident;
            let inner_code = parse_struct_fields(&strct.fields, name);
            quote! {
                #strct

                impl From<#name> for PathTree {
                    fn from(other_val: #name) -> Self {
                        let mut fields_vec: Vec<(String, Box<PathTree>)> = Vec::new();
                        #inner_code

                        PathTree::Path(fields_vec)
                    }
                }

                impl #name {
                    pub fn get_arguments(self, tx_signature: &str, instruction_idx: u8, inner_instructions_set: Option<u8>, program: &str) -> Vec<InstructionArgument> {
                        let path_tree: PathTree = self.into();

                        let mut instruction_arguments = Vec::new();

                        let mut instruction_arguments_mock = InstructionArgument::new(tx_signature, instruction_idx, inner_instructions_set, program);
                        path_tree.get_instruction_args_vec(&mut instruction_arguments, instruction_arguments_mock, &mut 0);

                        instruction_arguments
                    }
                }
            }
        }
        syn::Item::Enum(enm) => {
            let name = &enm.ident;
            let inner_code = parse_enum_variants(&enm.variants, name);

            let mut return_val = quote! {
                PathTree::Path(vec![(variant.to_string(), Box::new(PathTree::None)),
                    (variant.to_string(), Box::new(PathTree::Path(fields_vec)))])
            };

            if let Some(syn::NestedMeta::Meta(meta)) = attr.first() {
                let path = meta.path();

                if path.is_ident("InstrRoot") {
                    return_val = quote! {
                        PathTree::Path(fields_vec)
                    }
                }
            }
            quote! {
                #enm

                impl From<#name> for PathTree {
                    fn from(other_val: #name) -> Self {
                        let mut fields_vec = Vec::new();
                        let variant = #inner_code;

                        #return_val
                    }
                }

                impl #name {
                    pub fn get_arguments(self, tx_signature: &str, instruction_idx: u8, inner_instructions_set: Option<u8>, program: &str) -> Vec<InstructionArgument> {
                        let path_tree: PathTree = self.into();

                        let mut instruction_arguments = Vec::new();

                        let mut instruction_arguments_mock = InstructionArgument::new(tx_signature, instruction_idx, inner_instructions_set, program);
                        path_tree.get_instruction_args_vec(&mut instruction_arguments, instruction_arguments_mock, &mut 0);

                        instruction_arguments
                    }
                }
            }
        }
        _ => {
            unimplemented!("Only structs and enums are supported");
        }
    };

    trait_impl.into()
}

#[proc_macro_derive(HandleInstance)]
pub fn handle_instance(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let item = parse_macro_input!(item as syn::DeriveInput);
    let type_name = item.ident;
    let type_name_formatted = type_name.to_string().to_case(PROMETHEUS_CASE);

    quote! {

        impl Drop for #type_name {
            fn drop(&mut self) {
                log::debug!("{} has been dropped", stringify!(#type_name));
                crate::metrics_update!(dec total ACTIVE_HANDLE_INSTANCES_COUNT, &[#type_name_formatted]);
            }
        }

        impl Clone for #type_name {
            fn clone(&self) -> Self {
                crate::metrics_update!(inc total ACTIVE_HANDLE_INSTANCES_COUNT, &[#type_name_formatted]);
                Self {
                    sender: self.sender.clone(),
                }
            }
        }

    }
    .into()
}

#[proc_macro_derive(ActorInstance)]
pub fn actor_instance(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let item = parse_macro_input!(item as syn::DeriveInput);
    let type_name = item.ident;
    let type_name_formatted = type_name.to_string().to_case(PROMETHEUS_CASE);

    quote! {

        impl Drop for #type_name {
            fn drop(&mut self) {
                log::debug!("{} has been dropped", stringify!(#type_name));
                crate::metrics_update!(dec total ACTIVE_ACTOR_INSTANCES_COUNT, &[#type_name_formatted]);
            }
        }

    }
    .into()
}
