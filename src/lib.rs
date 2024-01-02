//! # bool-to-bitflags
//!
//! A struct attribute macro to pack structs with multiple boolean fields into efficent byte packing.
//!
//! This macro will make struct fields of type `bool` and `Option<bool>` be packed into a field called `__generated_flags`.
//!
//! This field is responsible for storing the packed bits, and should not be messed with manually, other than to initialize
//! as all-false with `{StructName}GeneratedFlags::empty()` or all-true with `{StructName}GeneratedFlags::all()`.
//!
//! ## Arguments
//! | Argument Name      | Type     | Default Value      | Description                                                                  |
//! |--------------------|----------|--------------------|------------------------------------------------------------------------------|
//! | `getter_prefix`    | `String` |                    | The prefix before getter names                                               |
//! | `setter_prefix`    | `String` | `set_`             | The prefix before setter names                                               |
//! | `private_getters`  | `bool`   | Field Visibility   | If true, getters are forced to be crate-private                              |
//! | `private_setters`  | `bool`   | Field Visibility   | If true, setters are forced to be crate-private                              |
//! | `document_setters` | `bool`   | `false`            | If true, field documentation is used for setters, instead of getters         |
//! | `owning_setters`   | `bool`   | `false`            | If true, setters take `self` and return `self` instead of taking `&mut self` |
use r#impl::bool_to_bitflags_impl;

mod args;
mod derive_hijack;
mod error;
mod r#impl;
mod impl_from_into;
mod impl_get_set;
mod strip_spans;

/// See [crate level](crate) documentation.
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
