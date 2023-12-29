use std::borrow::Cow;

use proc_macro2::{Ident, Span, TokenStream};
use quote::{format_ident, quote};

use crate::{
    args::Args,
    r#impl::{generate_pub_crate, ty_from_ident, BoolField},
};

fn extract_docs(attrs: &[syn::Attribute]) -> TokenStream {
    let attrs = attrs.iter().filter(|attr| attr.path().is_ident("doc"));
    quote!(#(#attrs)*)
}

fn handle_visibility_arg(field_vis: &syn::Visibility, private: bool) -> Cow<'_, syn::Visibility> {
    if private {
        Cow::Owned(generate_pub_crate())
    } else {
        Cow::Borrowed(field_vis)
    }
}

fn handle_owning_setters(owning_setters: bool) -> (TokenStream, syn::Type, Option<Ident>) {
    let self_ident = Ident::new("self", Span::call_site());
    if owning_setters {
        let owned_self_ty = ty_from_ident(Ident::new("Self", Span::call_site()));
        (quote!(mut self), owned_self_ty, Some(self_ident))
    } else {
        let unit_ret = syn::Type::Tuple(syn::TypeTuple {
            paren_token: syn::token::Paren::default(),
            elems: syn::punctuated::Punctuated::new(),
        });

        (quote!(&mut self), unit_ret, None)
    }
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

pub fn generate_getter_body(
    field: &BoolField,
    flag_field: &Ident,
    flags_name: &Ident,
) -> TokenStream {
    let flag_name = &field.flag_ident;
    match field {
        BoolField::Normal(..) => quote!(self.#flag_field.contains(#flags_name::#flag_name)),
        BoolField::Opt {
            tag_bit_flag_ident, ..
        } => quote!(
            if self.#flag_field.contains(#flags_name::#tag_bit_flag_ident) {
                Some(self.#flag_field.contains(#flags_name::#flag_name))
            } else {
                None
            }
        ),
    }
}

pub fn generate_getters_setters(
    struct_item: &syn::ItemStruct,
    flags_name: Ident,
    flag_field: Ident,
    bool_fields: &[BoolField],
    args: Args,
) -> TokenStream {
    let struct_name = &struct_item.ident;
    let (impl_generics, ty_generics, where_clause) = struct_item.generics.split_for_impl();

    let mut impl_body = TokenStream::new();
    for field in bool_fields {
        let field_docs = extract_docs(&field.attrs);
        let field_name = &field.field_ident;
        let flag_name = &field.flag_ident;

        let getter_vis = handle_visibility_arg(&field.vis, args.private_getters);
        let setter_vis = handle_visibility_arg(&field.vis, args.private_setters);
        let (setter_self_ty, setter_ret_ty, setter_ret) =
            handle_owning_setters(args.owning_setters);

        let (getter_name, setter_name) = args_to_names(&args, field_name);
        let (getter_docs, setter_docs) = if args.document_setters {
            (TokenStream::default(), field_docs)
        } else {
            let setter_docs = format!("Sets the {field_name} to the value provided.");
            (field_docs, quote!(#[doc = #setter_docs]))
        };

        let getter_body = generate_getter_body(field, &flag_field, &flags_name);
        let to_extend = match field {
            BoolField::Normal(_) => quote!(
                #getter_docs
                #getter_vis fn #getter_name(&self) -> bool {
                    #getter_body
                }

                #setter_docs
                #setter_vis fn #setter_name(#setter_self_ty, value: bool) -> #setter_ret_ty {
                    self.#flag_field.set(#flags_name::#flag_name, value);
                    #setter_ret
                }
            ),
            BoolField::Opt {
                tag_bit_flag_ident, ..
            } => quote!(
                #getter_docs
                #getter_vis fn #getter_name(&self) -> Option<bool> {
                    #getter_body
                }

                #setter_docs
                #setter_vis fn #setter_name(#setter_self_ty, value: Option<bool>) -> #setter_ret_ty {
                    if let Some(value) = value {
                        self.#flag_field.insert(#flags_name::#tag_bit_flag_ident);
                        self.#flag_field.set(#flags_name::#flag_name, value);
                    } else {
                        self.#flag_field.remove(#flags_name::#tag_bit_flag_ident);
                    };
                    #setter_ret
                }
            ),
        };

        impl_body.extend([to_extend])
    }

    quote!(
        impl #impl_generics #struct_name #ty_generics #where_clause {
            #impl_body
        }
    )
}
