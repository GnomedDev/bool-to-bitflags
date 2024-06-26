#![allow(clippy::manual_unwrap_or_default)] // Emitted inside darling's code
use darling::ast::NestedMeta;
use darling::FromMeta;
use proc_macro2::TokenStream;

use crate::error::Error;

/// Match documentation to crate level docs!!
#[derive(darling::FromMeta)]
#[allow(clippy::struct_excessive_bools)] // I would love to use bool_to_bitflags here... but
pub struct Args {
    pub getter_prefix: Option<String>,
    pub setter_prefix: Option<String>,
    #[darling(default)]
    pub private_getters: bool,
    #[darling(default)]
    pub private_setters: bool,
    #[darling(default)]
    pub document_setters: bool,
    #[darling(default)]
    pub owning_setters: bool,
}

impl Args {
    pub fn parse(args: TokenStream) -> Result<Self, Error> {
        Self::from_list(&NestedMeta::parse_meta_list(args)?).map_err(Error::Darling)
    }
}
