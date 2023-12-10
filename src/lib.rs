use proc_macro2::{Ident, Span};
use r#impl::bool_to_bitflags_impl;

mod derive_hijack;
mod error;
mod r#impl;
mod impl_from_into;
mod strip_spans;

fn field_to_flag_name(ident: &Ident) -> Ident {
    // Purposefully does not use field ident for flag name, to prevent flag
    // showing up in documentation/rust-analyzer hints
    Ident::new(&ident.to_string().to_uppercase(), Span::call_site())
}

#[proc_macro_attribute]
pub fn bool_to_bitflags(
    _: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let struct_item = syn::parse_macro_input!(item as syn::ItemStruct);
    match bool_to_bitflags_impl(struct_item) {
        Ok(output) => {
            #[cfg(feature = "procout")]
            procout::procout(&output, None, Some("output"));
            output.into()
        }
        Err(err) => err.into_compile_error().into(),
    }
}
