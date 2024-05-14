use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::{punctuated::Punctuated, Attribute, Field, Fields, FieldsNamed, ItemStruct, Token};

use crate::{
    impl_get_set::generate_getter_body,
    r#impl::{extract_cfgs, BoolField},
};

fn extract_fields(fields: &Fields) -> &Punctuated<Field, Token![,]> {
    if let Fields::Named(FieldsNamed { named, .. }) = fields {
        named
    } else {
        unreachable!()
    }
}

fn extract_passthrough_fields<'a>(
    fields: &'a Punctuated<Field, Token![,]>,
    flag_field_name: &'a Ident,
) -> impl Iterator<Item = (&'a Ident, impl Iterator<Item = &'a Attribute>)> {
    fields
        .iter()
        .map(|f| (f.ident.as_ref(), extract_cfgs(&f.attrs)))
        .filter_map(|(ident, cfgs)| ident.map(|ident| (ident, cfgs)))
        .filter(move |(ident, _)| *ident != flag_field_name)
}

pub fn impl_from(
    struct_item: &ItemStruct,
    original_struct_name: &Ident,
    flag_field_name: &Ident,
    flags_name: &Ident,
    bool_fields: &[BoolField],
) -> TokenStream {
    let struct_name = &struct_item.ident;
    let (impl_generics, ty_generics, where_clause) = struct_item.generics.split_for_impl();

    let fields = extract_fields(&struct_item.fields);
    let passthrough_fields = extract_passthrough_fields(fields, flag_field_name)
        .map(|(ident, cfgs)| quote!(#(#cfgs)* #ident: value.#ident));

    let flag_setters = bool_fields.iter().map(|field| {
        let flag_name = &field.flag_ident;
        let field_name = &field.field_ident;
        let cfgs = extract_cfgs(&field.attrs);
        let Some(tag_bit_flag_ident) = field.tag_bit_flag_ident() else {
            return quote!(
                #(#cfgs)*
                flags.set(#flags_name::#flag_name, value.#field_name);
            );
        };

        quote!(
            #(#cfgs)*
            if let Some(value) = value.#field_name {
                flags.insert(#flags_name::#tag_bit_flag_ident);
                flags.set(#flags_name::#flag_name, value);
            }
        )
    });

    quote!(
        impl #impl_generics From<#original_struct_name #ty_generics> for #struct_name #ty_generics #where_clause {
            fn from(value: #original_struct_name #ty_generics) -> Self {
                Self {
                    #(#passthrough_fields,)*
                    #flag_field_name: {
                        let mut flags = #flags_name::empty();
                        #(#flag_setters)*
                        flags
                    }
                }
            }
        }
    )
}

pub fn impl_into(
    struct_item: &ItemStruct,
    original_struct_name: &Ident,
    flag_field_name: &Ident,
    flags_name: &Ident,
    bool_fields: &[BoolField],
) -> TokenStream {
    let struct_name = &struct_item.ident;
    let (impl_generics, ty_generics, where_clause) = struct_item.generics.split_for_impl();

    let fields = extract_fields(&struct_item.fields);
    let passthrough_fields = extract_passthrough_fields(fields, flag_field_name)
        .map(|(ident, cfgs)| quote!(#(#cfgs)* #ident: self.#ident));

    let bool_fields = bool_fields.iter().map(|field| {
        let field_name = &field.field_ident;
        let cfgs = extract_cfgs(&field.attrs);
        let getter_body = generate_getter_body(field, flag_field_name, flags_name);

        quote!(#(#cfgs)* #field_name: #getter_body)
    });

    quote!(
        impl #impl_generics Into<#original_struct_name #ty_generics> for #struct_name #ty_generics #where_clause {
            fn into(self) -> #original_struct_name #ty_generics {
                #original_struct_name {
                    #(#bool_fields,)*
                    #(#passthrough_fields,)*
                }
            }
        }
    )
}
