use quote::quote;
use syn::*;
use proc_macro::TokenStream;

// Adapted from https://github.com/randomPoison/type-uuid
#[proc_macro_derive(Named, attributes(name))]
pub fn derive_type_name(input: TokenStream) -> TokenStream {
    type_name_derive(input)
}

pub(crate) fn type_name_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    // Construct a representation of Rust code as a syntax tree
    // that we can manipulate
    let ast: DeriveInput = syn::parse(input).unwrap();

    // Build the trait implementation
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

    if name.is_none() {
        panic!("No name given");
    }
    let name = name.unwrap();
    let gen = quote! {
        impl #impl_generics crate::named::Named for #type_name #type_generics #where_clause {
            fn name() -> &'static str {
                #name
            }
        }
    };
    gen.into()
}
