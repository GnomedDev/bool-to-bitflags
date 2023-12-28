use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote};
use syn::{Field, Fields, Ident, Token};

use crate::{
    args::Args,
    derive_hijack::{hijack_derives, HijackOutput},
    error::Error,
    field_to_flag_name,
    impl_from_into::{impl_from, impl_into},
    impl_get_set::generate_getters_setters,
    strip_spans::strip_spans,
};

fn path_from_ident(ident: Ident) -> syn::Path {
    syn::Path {
        leading_colon: None,
        segments: [syn::PathSegment {
            ident,
            arguments: syn::PathArguments::None,
        }]
        .into_iter()
        .collect(),
    }
}

fn generate_flag_field(flags_ident: Ident, field_ident: Ident) -> Field {
    Field {
        attrs: Vec::new(),
        ident: Some(field_ident),
        vis: syn::Visibility::Restricted(syn::VisRestricted {
            pub_token: <Token![pub]>::default(),
            paren_token: syn::token::Paren::default(),
            in_token: None,
            path: Box::new(path_from_ident(Ident::new("crate", Span::call_site()))),
        }),
        mutability: syn::FieldMutability::None,
        colon_token: Some(<Token![:]>::default()),
        ty: syn::Type::Path(syn::TypePath {
            qself: None,
            path: path_from_ident(flags_ident),
        }),
    }
}

fn is_bool_field(bool_fields: &mut Vec<Field>) -> impl FnMut(&Field) -> bool + '_ {
    let bool_ident = Ident::new("bool", Span::call_site());
    move |field| {
        if let syn::Type::Path(ty) = &field.ty {
            let segments = &ty.path.segments;
            if segments.first().is_some_and(|seg| seg.ident == bool_ident) {
                bool_fields.push(field.clone());
                return false;
            }
        }

        true
    }
}

fn extract_bool_fields(flag_field: Field, fields: Fields) -> Result<(Fields, Vec<Field>), Error> {
    let Fields::Named(mut fields) = fields else {
        return Err(Error::Custom(
            Span::call_site(),
            "Only structs with named fields are supported!",
        ));
    };

    let mut bool_fields = Vec::new();
    fields.named = fields
        .named
        .into_iter()
        .filter(is_bool_field(&mut bool_fields))
        .chain(std::iter::once(flag_field))
        .collect();

    Ok((Fields::Named(fields), bool_fields))
}

fn get_flag_size(bool_count: usize) -> TokenStream {
    match bool_count {
        0..=8 => quote!(u8),
        9..=16 => quote!(u16),
        17..=32 => quote!(u32),
        33..=64 => quote!(u64),
        65..=128 => quote!(u128),
        _ => panic!("Cannot fit {bool_count} bool fields into single bitflags type!"),
    }
}

fn generate_bitflags_type(
    flags_name: &Ident,
    bool_fields: &[Field],
    flags_derives: Vec<TokenStream>,
) -> TokenStream {
    let flags_size = get_flag_size(bool_fields.len());
    let flag_values = (0..bool_fields.len())
        .map(|i| (1 << i).to_string())
        .map(|i| syn::LitInt::new(&i, Span::call_site()));

    let flag_names = bool_fields
        .iter()
        .map(|f| f.ident.as_ref().unwrap())
        .map(field_to_flag_name);

    #[cfg(feature = "typesize")]
    let typesize_impl = Some(quote!(impl ::typesize::TypeSize for #flags_name {}));
    #[cfg(not(feature = "typesize"))]
    let typesize_impl: Option<TokenStream> = None;

    quote!(
        bitflags::bitflags! {
            #(#flags_derives)*
            pub(crate) struct #flags_name: #flags_size {
                #(const #flag_names = #flag_values;)*
            }
        }

        #typesize_impl
    )
}

pub fn bool_to_bitflags_impl(
    args: TokenStream,
    mut struct_item: syn::ItemStruct,
) -> Result<TokenStream, Error> {
    let args = Args::parse(args)?;

    // Hidden flags type should not have the span of the struct's name.
    let flags_name = format_ident!("{}Flags", struct_item.ident, span = Span::call_site());
    let flag_field_name = Ident::new("__generated_flags", Span::call_site());

    let mut original_struct = struct_item.clone();
    original_struct.ident = format_ident!("{}Original", original_struct.ident);
    strip_spans(&mut original_struct);

    let flag_field = generate_flag_field(flags_name.clone(), flag_field_name.clone());
    let (fields, bool_fields) = extract_bool_fields(flag_field, struct_item.fields)?;
    struct_item.fields = fields;

    let HijackOutput {
        compacted_struct_attrs,
        flags_derives,
    } = hijack_derives(&mut struct_item, &original_struct.ident)?;

    let from_impl = impl_from(
        &struct_item,
        &original_struct.ident,
        &flag_field_name,
        &flags_name,
        &bool_fields,
    );

    let into_impl = impl_into(
        &struct_item,
        &original_struct.ident,
        &flag_field_name,
        &flags_name,
        &bool_fields,
    );

    let bitflags_def = generate_bitflags_type(&flags_name, &bool_fields, flags_derives);
    let func_impls = generate_getters_setters(
        &struct_item,
        flags_name,
        flag_field_name,
        &bool_fields,
        args,
    );

    Ok(quote!(
        #[allow(clippy::struct_excessive_bools)]
        #original_struct
        #from_impl
        #into_impl

        #bitflags_def
        #(#compacted_struct_attrs)*
        #struct_item
        #func_impls
    ))
}
