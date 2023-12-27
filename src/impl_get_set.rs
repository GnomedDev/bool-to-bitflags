use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};
use syn::Field;

use crate::{args::Args, field_to_flag_name};

fn extract_docs(attrs: &[syn::Attribute]) -> TokenStream {
    let attrs = attrs.iter().filter(|attr| attr.path().is_ident("doc"));
    quote!(#(#attrs)*)
}

fn args_to_names(args: &Args, field_name: &Ident) -> (Ident, Ident) {
    let getter_prefix = args.getter_prefix.as_deref().unwrap_or("");
    let setter_prefix = args.setter_prefix.as_deref().unwrap_or("set_");
    let span = field_name.span();

    (
        format_ident!("{getter_prefix}{field_name}", span = span),
        format_ident!("{setter_prefix}{field_name}", span = span),
    )
}

pub fn generate_getters_setters(
    struct_item: &syn::ItemStruct,
    flags_name: Ident,
    flag_field: Ident,
    bool_fields: &[Field],
    args: Args,
) -> TokenStream {
    let struct_name = &struct_item.ident;
    let (impl_generics, ty_generics, where_clause) = struct_item.generics.split_for_impl();

    let mut impl_body = TokenStream::new();
    for field in bool_fields {
        let field_docs = extract_docs(&field.attrs);
        let field_name = field.ident.as_ref().unwrap();
        let flag_name = field_to_flag_name(field_name);

        let getter_vis = &field.vis;
        let setter_vis = if args.private_setters {
            &syn::Visibility::Inherited
        } else {
            getter_vis
        };

        let (getter_name, setter_name) = args_to_names(&args, field_name);
        let (getter_docs, setter_docs) = if args.document_setters {
            (TokenStream::default(), field_docs)
        } else {
            let setter_docs = format!("Sets the {field_name} to the value provided.");
            (field_docs, quote!(#[doc = #setter_docs]))
        };

        impl_body.extend([quote!(
            #getter_docs
            #getter_vis fn #getter_name(&self) -> bool {
                self.#flag_field.contains(#flags_name::#flag_name)
            }

            #setter_docs
            #setter_vis fn #setter_name(&mut self, value: bool) {
                self.#flag_field.set(#flags_name::#flag_name, value);
            }
        )]);
    }

    quote!(
        impl #impl_generics #struct_name #ty_generics #where_clause {
            #impl_body
        }
    )
}
