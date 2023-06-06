use proc_macro::TokenStream;
use proc_macro2::Span;
use syn::{parse::Parse, parse_macro_input, punctuated::Punctuated, DeriveInput, Path, Token};

#[derive(Clone, Default)]
struct TypeListArgs {
    types: Vec<Path>,
}

impl Parse for TypeListArgs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let types = Punctuated::<Path, Token![,]>::parse_terminated(input);
        match types {
            Ok(types) => Ok(TypeListArgs {
                types: types.into_iter().collect(),
            }),
            Err(_) => panic!("Failed to parse types: {}", input.to_string()),
        }
    }
}

fn create_registry_query(
    ro_args: Option<TypeListArgs>,
    rw_args: Option<TypeListArgs>,
) -> proc_macro2::TokenStream {
    // TODO: Optimize for single component case

    let mut rw_types = rw_args.clone().unwrap_or_default().types;
    let mut all_types = ro_args.clone().unwrap_or_default().types;
    all_types.append(&mut rw_types);

    let mut contains_quote = quote! {};
    for tp in &all_types {
        let ident = tp.get_ident();
        if contains_quote.is_empty() {
            contains_quote = quote! { reg.contains_component_from_iter::<#ident>(it) };
        } else {
            contains_quote =
                quote! { #contains_quote && reg.contains_component_from_iter::<#ident>(it) };
        }
    }

    let mut var_idx = 0;
    let mut fetch_quote = quote! {};
    let mut check_exists_quote = quote! {};
    let mut extract_ref_quote = quote! {};
    let mut extract_mut_quote = quote! {};

    let ro_count = ro_args.map(|ro| ro.types.len()).unwrap_or(0);
    let rw_count = rw_args.map(|rw| rw.types.len()).unwrap_or(0);

    for tp in &all_types {
        let ident = tp.get_ident();
        let var_name = syn::Ident::new(&format! {"var_{}", var_idx}, Span::call_site());

        if var_idx < ro_count {
            fetch_quote = quote! {
                #fetch_quote
                let #var_name = reg.get_component_ref_from_iter::<#ident>(it);
            };
        } else {
            fetch_quote = quote! {
                #fetch_quote
                let mut #var_name = reg.get_component_mut_from_iter::<#ident>(it);
            };
        }

        if var_idx == 0 {
            check_exists_quote = quote! {
                #check_exists_quote #var_name.is_some()
            };

            if var_idx < ro_count {
                extract_ref_quote = quote! {
                    unsafe { #var_name.unwrap_unchecked() }
                }
            } else {
                extract_mut_quote = quote! {
                    unsafe { #var_name.unwrap_unchecked() }
                }
            }
        } else {
            check_exists_quote = quote! {
                #check_exists_quote && #var_name.is_some()
            };

            if var_idx < ro_count {
                extract_ref_quote = quote! {
                    #extract_ref_quote, unsafe { #var_name.unwrap_unchecked() }
                };
            } else if var_idx == ro_count {
                extract_mut_quote = quote! {
                    unsafe { #var_name.unwrap_unchecked() }
                }
            } else {
                extract_mut_quote = quote! {
                    #extract_mut_quote, unsafe { #var_name.unwrap_unchecked() }
                };
            }
        }

        var_idx += 1;
    }

    if ro_count > 0 && rw_count == 0 {
        fetch_quote = quote! {
            #fetch_quote
            let exists = #check_exists_quote;
            if exists {
                Some((#extract_ref_quote))
            } else {
                None
            }
        };
    } else if ro_count == 0 && rw_count > 0 {
        fetch_quote = quote! {
            #fetch_quote
            let exists = #check_exists_quote;
            if exists {
                Some((#extract_mut_quote))
            } else {
                None
            }
        };
    } else if ro_count > 0 && rw_count > 0 {
        fetch_quote = quote! {
            #fetch_quote
            let exists = #check_exists_quote;
            if exists {
                Some(((#extract_ref_quote), (#extract_mut_quote)))
            } else {
                None
            }
        };
    } else {
        fetch_quote = quote! {
            #fetch_quote
            let exists = #check_exists_quote;
            if exists {
                Some(())
            } else {
                None
            }
        };
    }

    quote! {
        fn contains(it: tempest_ecs::registry::QueryIterator, reg: &'r tempest_ecs::registry::Registry) -> bool
        {
            #contains_quote
        }

        fn fetch(it: tempest_ecs::registry::QueryIterator, reg: &'r tempest_ecs::registry::Registry) -> Option<Self::Result>
        {
            #fetch_quote
        }
    }
}

pub fn derive_registry_query_impl(input: TokenStream) -> TokenStream {
    let tokens = parse_macro_input!(input as DeriveInput);
    let attrs = &tokens.attrs;
    let type_name = &tokens.ident;

    let readonly_type_list = attrs
        .iter()
        .filter(|attr| {
            attr.path().segments.len() == 1 && attr.path().segments[0].ident == "read_only"
        })
        .nth(0)
        .map(|attr| attr.parse_args::<TypeListArgs>().unwrap());

    let readwrite_type_list = attrs
        .iter()
        .filter(|attr| {
            attr.path().segments.len() == 1 && attr.path().segments[0].ident == "read_write"
        })
        .nth(0)
        .map(|attr| attr.parse_args::<TypeListArgs>().unwrap());

    let readonly_attr = readonly_type_list.as_ref().map(|args| {
        let mut list = quote! {};
        for elem in 0..args.types.len() {
            let tp = &args.types[elem];
            if elem == args.types.len() - 1 {
                list = quote! { #list &'r #tp };
            } else {
                list = quote! { #list &'r #tp, };
            }
        }

        list
    });

    let readwrite_attr = readwrite_type_list.as_ref().map(|args| {
        let mut list = quote! {};
        for elem in 0..args.types.len() {
            let tp = &args.types[elem];
            if elem == args.types.len() - 1 {
                list = quote! { #list &'r mut #tp };
            } else {
                list = quote! { #list &'r mut #tp, };
            }
        }

        list
    });

    let result_tokens = if readonly_attr.is_some() && readwrite_attr.is_some() {
        quote! {
            ((#readonly_attr), (#readwrite_attr))
        }
    } else if readonly_attr.is_some() && readwrite_attr.is_none() {
        quote! {
            (#readonly_attr)
        }
    } else if readonly_attr.is_none() && readwrite_attr.is_some() {
        quote! {
            (#readwrite_attr)
        }
    } else {
        quote! {()}
    };

    let generated = match tokens.data {
        syn::Data::Struct(_) => {
            let query = create_registry_query(readonly_type_list, readwrite_type_list);

            quote! {
                impl<'r> tempest_ecs::registry::RegistryQuery<'r> for #type_name {
                    type Result = #result_tokens;
                    
                    #query
                }
            }
        }
        syn::Data::Enum(_) => unimplemented!("Enum type not supported."),
        syn::Data::Union(_) => todo!("Union type not supported."),
    };

    generated.into()
}
