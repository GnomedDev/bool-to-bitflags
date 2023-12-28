use r#impl::bool_to_bitflags_impl;

mod args;
mod derive_hijack;
mod error;
mod r#impl;
mod impl_from_into;
mod impl_get_set;
mod strip_spans;

#[proc_macro_attribute]
pub fn bool_to_bitflags(
    args: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let struct_item = syn::parse_macro_input!(item as syn::ItemStruct);
    match bool_to_bitflags_impl(args.into(), struct_item) {
        Ok(output) => {
            #[cfg(feature = "procout")]
            procout::procout(&output, None, Some("output"));
            output.into()
        }
        Err(err) => err.into_compile_error().into(),
    }
}
