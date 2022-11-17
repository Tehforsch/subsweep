use proc_macro2::Literal;
use quote::{quote};
use syn::*;

// Adapted from https://github.com/randomPoison/type-uuid
#[proc_macro_derive(Named, attributes(name))]
pub fn derive_type_name(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    type_name_derive(input)
}

pub(crate) fn type_name_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast: DeriveInput = syn::parse(input).unwrap();

    let type_name = &ast.ident;

    let (impl_generics, type_generics, where_clause) = &ast.generics.split_for_impl();

    let mut name: Option<String> = None;
    for attribute in ast.attrs.iter().filter_map(|attr| attr.parse_meta().ok()) {
        let name_value = if let Meta::NameValue(name_value) = attribute {
            name_value
        } else {
            continue;
        };

        if name_value
            .path
            .get_ident()
            .map(|i| i != "name")
            .unwrap_or(true)
        {
            continue;
        }

        name = match name_value.lit {
            Lit::Str(lit_str) => Some(lit_str.value()),
            _ => panic!("`name` attribute must take the form `#[name = \"name\"]`."),
        };
    }

    let name = name.unwrap_or(ast.ident.to_string());

    let gen = quote! {
        impl #impl_generics derive_traits::Named for #type_name #type_generics #where_clause {
            fn name() -> &'static str {
                #name
            }
        }
    };
    gen.into()
}

#[proc_macro_attribute]
pub fn raxiom_parameters(args: proc_macro::TokenStream, input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    parameter_attr_derive(args, input)
}

pub(crate) fn parameter_attr_derive(args: proc_macro::TokenStream, input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let args: proc_macro2::TokenStream = args.into();
    let name: Option<Literal> = args.into_iter().next().map(|x| match x {
        proc_macro2::TokenTree::Literal(s) => s,
        _ => panic!("Unexpected token in parameter_section macro"),
    });
    
    let trait_impl: proc_macro2::TokenStream  = parameters_trait_impl(input.clone(), name).into();
    let input: proc_macro2::TokenStream = input.into();
    let output = quote! {
        #[derive(Clone, serde::Serialize, serde::Deserialize, bevy::prelude::Resource)]
        #[serde(deny_unknown_fields)]
        #[serde(rename_all = "snake_case")]
        #input

        #trait_impl
    };
    output.into()
}

pub(crate) fn parameters_trait_impl(input: proc_macro::TokenStream, section_name: Option<Literal>) -> proc_macro::TokenStream {
    let ast: DeriveInput = syn::parse(input).unwrap();
    let type_name = &ast.ident;
    let (impl_generics, type_generics, where_clause) = &ast.generics.split_for_impl();

    let gen = match section_name {
        Some(section_name) => quote! {
            impl #impl_generics ::derive_traits::RaxiomParameters for #type_name #type_generics #where_clause {
                fn section_name() -> Option<&'static str> {
                    Some(#section_name)
                }
            }
        },
        None => {
            quote! {
                impl #impl_generics ::derive_traits::RaxiomParameters for #type_name #type_generics #where_clause {
                    fn section_name() -> Option<&'static str> {
                        None
                    }
                }
            }
        }
    };
    gen.into()
}
