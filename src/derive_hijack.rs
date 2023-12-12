use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, ToTokens};
use syn::{punctuated::Punctuated, ItemStruct, Path, Token};

use crate::error::Error;

fn new_basic_segment(ident: &'static str) -> syn::PathSegment {
    syn::PathSegment {
        ident: syn::Ident::new(ident, Span::call_site()),
        arguments: syn::PathArguments::None,
    }
}

fn set_custom_impls(
    original_path: &str,
    serde_from: &mut Option<TokenStream>,
    serde_into: &mut Option<TokenStream>,
    derive_macros: Punctuated<Path, Token![,]>,
) -> Result<TokenStream, Error> {
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

        let Some(next_segment) = path_iter.next() else {
            filtered_derives.push(path);
            continue;
        };

        if next_segment == &serialize_segment {
            *serde_into = Some(quote!(#[serde(into = #original_path)]));
        } else if next_segment == &deserialize_segment {
            *serde_from = Some(quote!(#[serde(from = #original_path)]));
        } else {
            filtered_derives.push(path);
        }
    }

    Ok(quote!(#[derive(#(#filtered_derives),*)]))
}

pub struct HijackOutput {
    pub compacted_struct_attrs: Vec<TokenStream>,
    pub flags_derives: Vec<TokenStream>,
}

pub fn hijack_derives(
    compacted_struct: &mut ItemStruct,
    original_mod_name: &Ident,
) -> Result<HijackOutput, Error> {
    let original_path = format!("{original_mod_name}::{}", compacted_struct.ident);

    let mut serde_from = None;
    let mut serde_into = None;
    let mut new_attrs = Vec::new();
    for attr in &compacted_struct.attrs {
        if attr.path().is_ident("derive") {
            let parser = Punctuated::<Path, Token![,]>::parse_terminated;
            new_attrs.push(set_custom_impls(
                &original_path,
                &mut serde_from,
                &mut serde_into,
                attr.parse_args_with(parser)?,
            )?);

            continue;
        }

        new_attrs.push(attr.to_token_stream())
    }

    let compacted_attrs = compacted_struct
        .attrs
        .drain(..)
        .map(|a| a.to_token_stream())
        .chain(serde_from)
        .chain(serde_into)
        .collect();

    Ok(HijackOutput {
        compacted_struct_attrs: compacted_attrs,
        flags_derives: new_attrs,
    })
}
