use proc_macro::TokenStream;
use quote::{quote};
use syn::*;

// Adapted from https://github.com/randomPoison/type-uuid
#[proc_macro_derive(Named, attributes(name))]
pub fn derive_type_name(input: TokenStream) -> TokenStream {
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
        impl #impl_generics named::Named for #type_name #type_generics #where_clause {
            fn name() -> &'static str {
                #name
            }
        }
    };
    gen.into()
}

#[proc_macro_attribute]
pub fn parameter_section(args: TokenStream, input: TokenStream) -> TokenStream {
    section_derive(args, input)
}

pub(crate) fn section_derive(args: TokenStream, input: TokenStream) -> proc_macro::TokenStream {
    let args: proc_macro2::TokenStream = args.into();
    let name: String = args.into_iter().next().unwrap_or_else(|| panic!("Section name not specified, try #[section_name(\"...\")]")).to_string();
    
    let input: proc_macro2::TokenStream = input.into();
    let output = quote! {
        #[derive(Clone, crate::parameter_plugin::RaxiomParameters, Serialize, Deserialize)]
        #[serde(deny_unknown_fields)]
        #[section_name = #name]
        #input
    };
    output.into()
}

#[proc_macro_derive(RaxiomParameters, attributes(section_name))]
pub fn derive_parameters(input: TokenStream) -> TokenStream {
    parameters_derive(input)
}

pub(crate) fn parameters_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast: DeriveInput = syn::parse(input).unwrap();
    let type_name = &ast.ident;
    let (impl_generics, type_generics, where_clause) = &ast.generics.split_for_impl();
    let name: Option<String> = ast
        .attrs
        .iter()
        .filter_map(|attr| attr.parse_meta().ok())
        .filter_map(|attr| -> Option<String> {
            if let Meta::NameValue(name_value) = attr {
                if let Some(ident) = name_value.path.get_ident() {
                    if ident == "section_name" {
                        return match name_value.lit {
                            Lit::Str(lit_str) => Some(lit_str.value()),
                            _ => panic!(
                                "`section_name` attribute must take the form `#[section_name = \"name\"]`."
                            ),
                        };
                    }
                }
            }
            None
        })
        .next();

    let gen = match name {
        Some(name) => {
            quote! {
                impl #impl_generics crate::parameter_plugin::RaxiomParameters for #type_name #type_generics #where_clause {
                    fn section_name() -> Option<&'static str> {
                        Some(#name)
                    }
                }
            }
        },
        None => {
            quote! {
                impl #impl_generics crate::parameter_plugin::RaxiomParameters for #type_name #type_generics #where_clause {
                    fn section_name() -> Option<&'static str> {
                        None
                    }
                }
            }
        }
    };
    gen.into()
}
