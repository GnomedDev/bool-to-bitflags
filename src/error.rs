use std::borrow::Cow;

use proc_macro2::{Span, TokenStream};

pub enum Error {
    Syn(syn::Error),
    Darling(darling::Error),
    Custom(Span, Cow<'static, str>),
}

impl Error {
    pub fn into_compile_error(self) -> TokenStream {
        match self {
            Self::Syn(err) => err.to_compile_error(),
            Self::Darling(err) => err.write_errors(),
            Self::Custom(span, err) => syn::Error::new(span, err).to_compile_error(),
        }
    }
}

impl From<syn::Error> for Error {
    fn from(value: syn::Error) -> Self {
        Self::Syn(value)
    }
}
