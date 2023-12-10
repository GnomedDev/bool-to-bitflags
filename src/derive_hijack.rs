use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, ToTokens};
use syn::{punctuated::Punctuated, Field, ItemStruct, Path, Token};

use crate::{
    error::Error,
    impl_from_into::{impl_from, impl_into},
};

bitflags::bitflags! {
    #[derive(Clone, Copy)]
    pub struct DerivedTraits: u8 {
        const SERDE_DE  = 0b01;
        const SERDE_SER = 0b10;
    }
}

fn new_basic_segment(ident: &'static str) -> syn::PathSegment {
    syn::PathSegment {
        ident: syn::Ident::new(ident, Span::call_site()),
        arguments: syn::PathArguments::None,
    }
}

fn set_custom_impls(
    struct_item: &ItemStruct,
    flag_field_name: &Ident,
    flags_name: &Ident,
    bool_fields: &[Field],
    should_impl: &mut DerivedTraits,
    derive_macros: Punctuated<Path, Token![,]>,
) -> Result<(TokenStream, Option<TokenStream>, Option<TokenStream>), Error> {
    let serde_segment = new_basic_segment("serde");
    let serialize_segment = new_basic_segment("Serialize");
    let deserialize_segment = new_basic_segment("Deserialize");

    let mut filtered_derives = Vec::new();
    for path in &derive_macros {
        let mut path_iter = path.segments.iter();
        let Some(first_segment) = path_iter.next() else {
            continue;
        };

        if first_segment != &serde_segment {
            filtered_derives.push(path);
            continue;
        }

        if let Some(next_segment) = path_iter.next() {
            if next_segment == &serialize_segment {
                should_impl.insert(DerivedTraits::SERDE_SER);
                continue;
            } else if next_segment == &deserialize_segment {
                should_impl.insert(DerivedTraits::SERDE_DE);
                continue;
            }
        }

        filtered_derives.push(dbg!(path));
    }

    let serde_from = should_impl
        .contains(DerivedTraits::SERDE_DE)
        .then(|| impl_from(struct_item, flag_field_name, flags_name, bool_fields));

    let serde_into = should_impl
        .contains(DerivedTraits::SERDE_SER)
        .then(|| impl_into(struct_item, flag_field_name, bool_fields));

    Ok((
        quote!(#[derive(#(#filtered_derives,)*)]),
        serde_from,
        serde_into,
    ))
}

pub struct HijackOutput {
    pub compacted_struct_attrs: Vec<TokenStream>,
    pub flags_derives: Vec<TokenStream>,
    pub from_into_impls: TokenStream,
}

pub fn hijack_derives(
    compacted_struct: &mut ItemStruct,
    flag_field_name: &Ident,
    original_mod_name: &Ident,
    flags_name: &Ident,
    bool_fields: &[Field],
) -> Result<HijackOutput, Error> {
    let mut should_impl = DerivedTraits::empty();
    let original_path = format!("{original_mod_name}::{}", compacted_struct.ident);

    let mut serde_from = None;
    let mut serde_into = None;
    let mut new_attrs = Vec::new();
    let mut compacted_attrs = Vec::new();
    for attr in &compacted_struct.attrs {
        if attr.path().is_ident("derive") {
            let parser = Punctuated::<Path, Token![,]>::parse_terminated;
            let (new_derive, new_from, new_into) = set_custom_impls(
                compacted_struct,
                flag_field_name,
                flags_name,
                bool_fields,
                &mut should_impl,
                attr.parse_args_with(parser)?,
            )?;

            new_attrs.push(new_derive);

            if let Some(new_from) = new_from {
                serde_from = Some(new_from);
                compacted_attrs.push(quote!(#[serde(from = #original_path)]));
            }
            if let Some(new_into) = new_into {
                serde_into = Some(new_into);
                compacted_attrs.push(quote!(#[serde(into = #original_path)]));
            }

            continue;
        }

        new_attrs.push(attr.to_token_stream())
    }

    Ok(HijackOutput {
        from_into_impls: quote!(#serde_from #serde_into),
        compacted_struct_attrs: compacted_attrs,
        flags_derives: new_attrs,
    })
}
