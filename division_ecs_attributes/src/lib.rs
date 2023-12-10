use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(Component)]
pub fn derive_component(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let is_supported = match input.data {
        syn::Data::Struct(s) => s.fields.len() > 0,
        _ => false,
    };

    if !is_supported {
        panic!("Components can be non-empty structs only");
    }

    let type_name = input.ident.to_string();
    format!("impl Component for {type_name} {{}}")
        .parse()
        .unwrap()
}

#[proc_macro_derive(Tag)]
pub fn derive_tag(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let is_supported = match input.data {
        syn::Data::Struct(s) => s.fields.len() == 0,
        _ => false
    };

    if !is_supported {
        panic!("Tags can be empty structs only");
    }

    let type_name = input.ident.to_string();
    format!("impl Tag for {type_name} {{}}")
        .parse()
        .unwrap()
}
