use proc_macro2::Span;
use syn::{GenericParam, ItemStruct};

// Strip spans from the input copy to prevent weirdness.
pub fn strip_spans(struct_item: &mut ItemStruct) {
    struct_item.ident.set_span(Span::call_site());
    for generic in &mut struct_item.generics.params {
        match generic {
            GenericParam::Const(c) => c.ident.set_span(Span::call_site()),
            GenericParam::Lifetime(l) => l.lifetime.ident.set_span(Span::call_site()),
            GenericParam::Type(t) => t.ident.set_span(Span::call_site()),
        }
    }

    for field in &mut struct_item.fields {
        if let Some(ident) = &mut field.ident {
            ident.set_span(Span::call_site())
        }
    }
}
