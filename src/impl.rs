use std::borrow::Cow;

use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote};
use syn::{Field, Fields, Ident, Token};

use crate::{
    args::Args,
    derive_hijack::{hijack_derives, HijackOutput},
    error::Error,
    impl_from_into::{impl_from, impl_into},
    impl_get_set::generate_getters_setters,
    strip_spans::strip_spans,
};

pub struct BoolFieldInner {
    pub field_ident: Ident,
    pub flag_ident: Ident,
    pub attrs: Vec<syn::Attribute>,
    pub vis: syn::Visibility,
}

pub enum BoolField {
    Normal(BoolFieldInner),
    Opt {
        bool_bit: BoolFieldInner,
        tag_bit_flag_ident: Ident,
    },
}

impl BoolField {
    fn from_field(field: &Field) -> Self {
        let field_ident = field.ident.clone().unwrap();
        BoolField::Normal(BoolFieldInner {
            flag_ident: Ident::new(&field_ident.to_string().to_uppercase(), Span::call_site()),
            attrs: field.attrs.clone(),
            vis: field.vis.clone(),
            field_ident,
        })
    }

    fn from_opt_bool_field(field: &Field) -> Self {
        match Self::from_field(field) {
            BoolField::Opt { .. } => unreachable!(),
            BoolField::Normal(bool_bit) => BoolField::Opt {
                tag_bit_flag_ident: format_ident!("{}_OPT_TAG", bool_bit.flag_ident),
                bool_bit,
            },
        }
    }

    pub fn tag_bit_flag_ident(&self) -> Option<&Ident> {
        match self {
            BoolField::Normal(_) => None,
            BoolField::Opt {
                tag_bit_flag_ident, ..
            } => Some(tag_bit_flag_ident),
        }
    }
}

impl std::ops::Deref for BoolField {
    type Target = BoolFieldInner;
    fn deref(&self) -> &Self::Target {
        match self {
            BoolField::Normal(inner) => inner,
            BoolField::Opt { bool_bit, .. } => bool_bit,
        }
    }
}

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

pub fn generate_pub_crate() -> syn::Visibility {
    syn::Visibility::Restricted(syn::VisRestricted {
        pub_token: <Token![pub]>::default(),
        paren_token: syn::token::Paren::default(),
        in_token: None,
        path: Box::new(path_from_ident(Ident::new("crate", Span::call_site()))),
    })
}

pub fn ty_from_ident(ident: syn::Ident) -> syn::Type {
    let path = path_from_ident(ident);
    syn::Type::Path(syn::TypePath { qself: None, path })
}

fn generate_flag_field(flags_ident: Ident, field_ident: Ident) -> Field {
    Field {
        attrs: Vec::new(),
        ident: Some(field_ident),
        vis: generate_pub_crate(),
        mutability: syn::FieldMutability::None,
        colon_token: Some(<Token![:]>::default()),
        ty: ty_from_ident(flags_ident),
    }
}

fn generate_generic(ty: syn::Type) -> syn::PathArguments {
    syn::PathArguments::AngleBracketed(syn::AngleBracketedGenericArguments {
        colon2_token: None,
        lt_token: <Token![<]>::default(),
        args: [syn::GenericArgument::Type(ty)].into_iter().collect(),
        gt_token: <Token![>]>::default(),
    })
}

fn is_bool_field(bool_fields: &mut Vec<BoolField>) -> impl FnMut(&Field) -> bool + '_ {
    let bool_ident = Ident::new("bool", Span::call_site());
    let opt_ident = Ident::new("Option", Span::call_site());
    let bool_generic = generate_generic(ty_from_ident(bool_ident.clone()));

    move |field| {
        if let syn::Type::Path(ty) = &field.ty {
            let segments = &ty.path.segments;
            let first_seg = segments.first().expect("field type path has one segment");

            if first_seg.ident == opt_ident && first_seg.arguments == bool_generic {
                bool_fields.push(BoolField::from_opt_bool_field(field));
            } else if first_seg.ident == bool_ident {
                bool_fields.push(BoolField::from_field(field));
            } else {
                return true;
            }

            return false;
        }

        true
    }
}

fn extract_bool_fields(
    flag_field: Field,
    fields: Fields,
) -> Result<(Fields, Vec<BoolField>), Error> {
    let Fields::Named(mut fields) = fields else {
        return Err(Error::Custom(
            Span::call_site(),
            Cow::Borrowed("bool_to_bitflags: Only structs with named fields are supported!"),
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

fn get_flag_size(bool_count: usize) -> Result<syn::Type, Error> {
    let ty_name = match bool_count {
        0..=8 => "u8",
        9..=16 => "u16",
        17..=32 => "u32",
        33..=64 => "u64",
        65..=128 => "u128",
        _ => {
            let err_msg = format!(
                "bool_to_bitflags: Cannot fit {bool_count} bool fields into single bitflags type!"
            );

            return Err(Error::Custom(Span::call_site(), Cow::Owned(err_msg)));
        }
    };

    Ok(ty_from_ident(Ident::new(ty_name, Span::call_site())))
}

fn generate_bitflags_type(
    flags_name: &Ident,
    flags_size: syn::Type,
    bool_fields: &[BoolField],
    flags_derives: Vec<TokenStream>,
) -> TokenStream {
    let opt_bools = bool_fields.iter().filter_map(|f| f.tag_bit_flag_ident());
    let flag_values = (0..(bool_fields.len() + opt_bools.clone().count()))
        .map(|i| (1 << i).to_string())
        .map(|i| syn::LitInt::new(&i, Span::call_site()));

    let flag_names = bool_fields.iter().map(|f| &f.flag_ident).chain(opt_bools);

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
    let flag_field_name = Ident::new("__generated_flags", Span::call_site());
    let flags_name = format_ident!(
        "{}GeneratedFlags",
        struct_item.ident,
        span = Span::call_site()
    );

    let mut original_struct = struct_item.clone();
    original_struct.ident = format_ident!("{}GeneratedOriginal", original_struct.ident);
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

    let flags_size = get_flag_size(bool_fields.len())?;
    let bitflags_def = generate_bitflags_type(&flags_name, flags_size, &bool_fields, flags_derives);
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
