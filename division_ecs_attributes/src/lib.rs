use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(Component)]
pub fn derive_component(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let type_name = input.ident.to_string();

    format!("impl Component for {type_name} {{}}").parse().unwrap()
}