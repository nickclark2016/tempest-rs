use std::sync::atomic::{AtomicUsize, Ordering};

use proc_macro::TokenStream;
use syn::{
    parse::Parse, parse_macro_input, Data, DeriveInput, Fields, LitInt, Token, Type, TypePath,
};

static TYPE_IDS: AtomicUsize = AtomicUsize::new(0);

pub fn derive_component_impl(input: TokenStream) -> TokenStream {
    let ast: DeriveInput = syn::parse(input).unwrap();

    // Get the name of the type being derived for
    let type_name = &ast.ident;

    // Generate the type ID for this type
    let type_id = TYPE_IDS.fetch_add(1, Ordering::SeqCst);

    // Generate the implementation of the TypeId trait
    let gen = match ast.data {
        Data::Struct(ref data) => match data.fields {
            Fields::Named(_) => {
                quote! {
                    impl Component for #type_name {
                        fn id() -> usize {
                            #type_id
                        }
                    }

                    impl Clone for #type_name {
                        fn clone(&self) -> Self {
                            *self
                        }
                    }

                    impl Copy for #type_name {}
                }
            }
            Fields::Unnamed(_) => {
                quote! {
                    impl Component for #type_name {
                        fn id() -> usize {
                            #type_id
                        }
                    }

                    impl Clone for #type_name {
                        fn clone(&self) -> Self {
                            *self
                        }
                    }

                    impl Copy for #type_name {}
                }
            }
            Fields::Unit => {
                quote! {
                    impl Component for #type_name {
                        fn id() -> usize {
                            #type_id
                        }
                    }

                    impl Clone for #type_name {
                        fn clone(&self) -> Self {
                            *self
                        }
                    }

                    impl Copy for #type_name {}
                }
            }
        },
        Data::Enum(_) => panic!("Cannot derive TypeId for enums"),
        Data::Union(_) => panic!("Cannot derive TypeId for unions"),
    };

    // Return the generated implementation
    gen.into()
}

struct ComponentTupleNthElementInput {
    tp: Type,
    index: LitInt,
}

impl Parse for ComponentTupleNthElementInput {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let tp = input.parse()?;
        input.parse::<Token![,]>().unwrap();
        let index = input.parse()?;

        Ok(Self { tp, index })
    }
}

pub fn component_tuple_nth_element_impl(input: TokenStream) -> TokenStream {
    let elem_input = parse_macro_input!(input as ComponentTupleNthElementInput);
    let index_requested = elem_input.index.base10_parse::<usize>().unwrap();
    let generated = match elem_input.tp {
        Type::Path(path) => component_tuple_nth_element_path_impl(path, index_requested),
        Type::Tuple(tup) => {
            let n = index_requested;
            let text = &tup.elems[n];
            quote! {
                #text
            }
        }
        _ => unimplemented!("Not a Tuple Type"),
    };

    generated.into()
}

fn component_tuple_nth_element_path_impl(tup: TypePath, elem: usize) -> proc_macro2::TokenStream {
    let name = tup.path.get_ident();
    match name {
        Some(name) => {
            if elem == 0 {
                quote!(<#name as tempest_ecs::component::ComponentTuple>::Head)
            } else {
                let mut rest = quote!(<#name as tempest_ecs::component::ComponentTuple>::Rest);

                for _ in 1..elem {
                    rest = quote! {
                        <#rest as tempest_ecs::component::ComponentTuple>::Rest
                    }
                }

                let head = quote! {
                    <#rest as tempest_ecs::component::ComponentTuple>::Head
                };

                head
            }
        }
        None => {
            let segments = tup
                .path
                .segments
                .iter()
                .fold(String::new(), |a, b| a + "::" + &b.ident.to_string());
            panic!("Failed to parse identity for rest of {}", segments);
        }
    }
}
