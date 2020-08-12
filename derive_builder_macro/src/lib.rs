//! Derive a builder for a struct

#![crate_type = "proc-macro"]
#![deny(warnings)]

#[macro_use]
extern crate darling;

extern crate proc_macro;
extern crate proc_macro2;
#[macro_use]
extern crate syn;
#[macro_use]
extern crate quote;
#[cfg(feature = "logging")]
#[macro_use]
extern crate log;
extern crate derive_builder_core;
#[cfg(feature = "logging")]
extern crate env_logger;

#[cfg(not(feature = "logging"))]
#[macro_use]
mod log_disabled;
mod options;

use darling::FromDeriveInput;
use options::Options;
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

    builder_for_struct(ast).into()
}

fn builder_for_struct(ast: syn::DeriveInput) -> proc_macro2::TokenStream {
    debug!("Deriving Builder for `{}`.", ast.ident);

    let opts = match Options::from_derive_input(&ast) {
        Ok(val) => val,
        Err(err) => {
            return err.write_errors();
        }
    };

    let mut builder = opts.as_builder();
    let mut build_fn = opts.as_build_method();

    builder.doc_comment(format!(
        include_str!("doc_tpl/builder_struct.md"),
        struct_name = ast.ident
    ));
    build_fn.doc_comment(format!(
        include_str!("doc_tpl/builder_method.md"),
        struct_name = ast.ident
    ));

    for field in opts.fields() {
        builder.push_field(field.as_builder_field());
        builder.push_setter_fn(field.as_setter());
        build_fn.push_initializer(field.as_initializer());
    }

    builder.push_build_fn(build_fn);

    quote!(#builder)
}
