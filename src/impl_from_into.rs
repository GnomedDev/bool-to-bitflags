use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::{punctuated::Punctuated, Field, Fields, FieldsNamed, ItemStruct, Token};

use crate::field_to_flag_name;

fn extract_fields(fields: &Fields) -> &Punctuated<Field, Token![,]> {
    if let Fields::Named(FieldsNamed { named, .. }) = fields {
        named
    } else {
        unreachable!()
    }
}

pub fn impl_from(
    struct_item: &ItemStruct,
    flag_field_name: &Ident,
    flags_name: &Ident,
    bool_fields: &[Field],
) -> TokenStream {
    let struct_name = &struct_item.ident;
    let (impl_generics, ty_generics, where_clause) = struct_item.generics.split_for_impl();

    let fields = extract_fields(&struct_item.fields);
    let passthrough_fields = fields
        .iter()
        .filter_map(|f| f.ident.as_ref())
        .filter(|ident| *ident != flag_field_name)
        .map(|ident| quote!(#ident: value.#ident));

    let bool_fields = bool_fields.iter().filter_map(|f| f.ident.as_ref());
    let bool_fields_upper = bool_fields.clone().map(field_to_flag_name);

    quote!(
        impl #impl_generics From<#struct_name #ty_generics> for super::#struct_name #ty_generics #where_clause {
            fn from(value: #struct_name #ty_generics) -> Self {
                Self {
                    #(#passthrough_fields,)*
                    #flag_field_name: {
                        let mut flags = super::#flags_name::empty();
                        #(
                            if value.#bool_fields {
                                flags |= super::#flags_name::#bool_fields_upper;
                            }
                        )*
                        flags
                    }
                }
            }
        }
    )
}

pub fn impl_into(
    struct_item: &ItemStruct,
    flag_field_name: &Ident,
    bool_fields: &[Field],
) -> TokenStream {
    let struct_name = &struct_item.ident;
    let (impl_generics, ty_generics, where_clause) = struct_item.generics.split_for_impl();

    let fields = extract_fields(&struct_item.fields);
    let passthrough_fields = fields
        .iter()
        .filter_map(|f| f.ident.as_ref())
        .filter(|ident| *ident != flag_field_name)
        .map(|ident| quote!(#ident: self.#ident));

    let bool_fields = bool_fields
        .iter()
        .map(|f| &f.ident)
        .map(|ident| quote!(#ident: self.#ident()));

    quote!(
        impl #impl_generics Into<#struct_name #ty_generics> for super::#struct_name #ty_generics #where_clause {
            fn into(self) -> #struct_name #ty_generics {
                #struct_name {
                    #(#passthrough_fields,)*
                    #(#bool_fields,)*
                }
            }
        }
    )
}
