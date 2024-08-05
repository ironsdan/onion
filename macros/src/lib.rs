extern crate proc_macro;
extern crate quote;

use proc_macro::TokenStream;
use quote::quote;
use syn::DeriveInput;

#[proc_macro_derive(Resource)]
pub fn derive_resource(item: TokenStream) -> TokenStream {
    let DeriveInput { ident, .. } = syn::parse_macro_input!(item as syn::DeriveInput);

    quote!(
        impl Resource for #ident {}
    )
    .into()
}

#[proc_macro_derive(Event)]
pub fn derive_event(item: TokenStream) -> TokenStream {
    let DeriveInput { ident, .. } = syn::parse_macro_input!(item as syn::DeriveInput);

    quote!(
        impl Event for #ident {}
        impl Resource for #ident {}
    )
    .into()
}
