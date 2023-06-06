use component::component_tuple_nth_element_impl;
use proc_macro::TokenStream;
use registry::derive_registry_query_impl;
use syn::{parse_macro_input, Type};

mod component;
mod registry;

#[macro_use]
extern crate quote;

#[proc_macro_derive(Component)]
pub fn derive_component(input: TokenStream) -> TokenStream {
    component::derive_component_impl(input)
}

#[proc_macro]
pub fn component_tuple_arity(input: TokenStream) -> TokenStream {
    let tp = parse_macro_input!(input as Type);
    let generated = match tp {
        Type::Path(path) => {
            let name = path.path.get_ident().unwrap();
            quote! {
                <#name as ::tempest::ecs::component::ComponentTuple>::ARITY
            }
        }
        Type::Tuple(tup) => {
            let size = tup.elems.len();
            quote!(#size)
        }
        _ => unimplemented!("Not a Tuple Type"),
    };

    generated.into()
}

#[proc_macro]
pub fn component_tuple_nth_element(input: TokenStream) -> TokenStream {
    component_tuple_nth_element_impl(input)
}

#[proc_macro_derive(RegistryQuery, attributes(read_only, read_write))]
pub fn derive_registry_query(input: TokenStream) -> TokenStream {
    derive_registry_query_impl(input)
}
