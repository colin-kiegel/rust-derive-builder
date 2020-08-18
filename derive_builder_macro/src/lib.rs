//! Derive a builder for a struct

#![crate_type = "proc-macro"]
#![deny(warnings)]

extern crate proc_macro;
#[macro_use]
extern crate syn;
extern crate derive_builder_core;
#[cfg(feature = "logging")]
extern crate env_logger;

use proc_macro::TokenStream;
#[cfg(feature = "logging")]
use std::sync::Once;

#[cfg(feature = "logging")]
static INIT_LOGGER: Once = Once::new();

#[doc(hidden)]
#[proc_macro_derive(Builder, attributes(builder))]
pub fn derive(input: TokenStream) -> TokenStream {
    #[cfg(feature = "logging")]
    INIT_LOGGER.call_once(|| {
        env_logger::init();
    });

    let ast = parse_macro_input!(input as syn::DeriveInput);

    derive_builder_core::builder_for_struct(ast).into()
}
