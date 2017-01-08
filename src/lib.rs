//! Derive a builder for a struct
//!
//! This crate implements the _non-consuming_ [builder pattern].
//! When applied to a struct, it will derive **setter-methods** for all struct fields.
//!
//! **Please note**:
//!
//! * There are slightly different ways to implement the builder pattern in rust.
//!   The preferred way to do it, is the so called _non-consuming_ variant.
//!   That means: all generated setter-methods take and return `&mut self`.
//! * To complete the builder pattern you only have to implement at least one method
//!   which actually builds something based on the struct.
//!   These custom build methods of yours should also take `&mut self` to take advantage of the
//!   non-consuming pattern.
//! * **Don't worry at all** if you have to `clone` or `copy` data in your build methods,
//!   because luckily the Compiler is smart enough to optimize them away in release builds
//!   for your every-day use cases. Thats quite a safe bet - we checked this for you. ;-)
//!   Switching to consuming signatures (=`self`) would not give you any performance
//!   gain, but only restrict your API for every-day use cases
//!
//! [builder pattern]: https://aturon.github.io/ownership/builders.html
//!
//! # Examples
//!
//! ```rust
//! #[macro_use] extern crate derive_builder;
//!
//! #[derive(Debug, PartialEq, Default, Clone, Builder)]
//! struct Lorem {
//!     ipsum: String,
//!     dolor: i32,
//! }
//!
//! fn main() {
//!     let x = Lorem::default().ipsum("sit").dolor(42).clone();
//!     assert_eq!(x, Lorem { ipsum: "sit".into(), dolor: 42 });
//! }
//! ```
//!
//! In `main()`: The final call of `clone()` represents the act of **building a new struct**
//! when our builder is ready. For the sake of brevity we chose clone and pretend we get
//! something brand new. As already mentioned, the compiler will optimize this away in release
//! mode.
//!
//! ## Generic structs
//!
//! ```rust
//! #[macro_use] extern crate derive_builder;
//!
//! #[derive(Debug, PartialEq, Default, Clone, Builder)]
//! struct GenLorem<T> {
//!     ipsum: String,
//!     dolor: T,
//! }
//!
//! fn main() {
//!     let x = GenLorem::default().ipsum("sit").dolor(42).clone();
//!     assert_eq!(x, GenLorem { ipsum: "sit".into(), dolor: 42 });
//! }
//! ```
//!
//! ## Doc-Comments and Attributes
//!
//! `#[derive(Builder)]` copies doc-comments and attributes `#[...]` from your fields
//! to the according setter-method, if it is one of the following:
//!
//! * `/// ...`
//! * `#[doc = ...]`
//! * `#[cfg(...)]`
//! * `#[allow(...)]`
//!
//! ```rust
//! #[macro_use] extern crate derive_builder;
//!
//! #[derive(Builder)]
//! struct Lorem {
//!     /// `ipsum` may be any `String` (be creative).
//!     ipsum: String,
//!     #[doc = r"`dolor` is the estimated amount of work."]
//!     dolor: i32,
//!     // `#[derive(Builder)]` understands conditional compilation via cfg-attributes,
//!     // i.e. => "no field = no setter".
//!     #[cfg(target_os = "macos")]
//!     #[allow(non_snake_case)]
//!     Im_a_Mac: bool,
//! }
//! # fn main() {}
//! ```
//!
//! ## Gotchas
//!
//! - Tuple structs and unit structs are not supported as they have no field
//!   names.
//! - When defining a generic struct, you cannot use `VALUE` as a generic
//!   parameter as this is what all setters are using.

#![crate_type = "proc-macro"]

extern crate proc_macro;
extern crate syn;
#[macro_use]
extern crate quote;
#[macro_use]
extern crate log;
extern crate env_logger;

mod options;

use proc_macro::TokenStream;
use options::{Options, SetterPattern};

#[proc_macro_derive(Builder, attributes(setters, getters, setter, getter))]
pub fn derive(input: TokenStream) -> TokenStream {
    env_logger::init().unwrap();

    let input = input.to_string();

    let ast = syn::parse_macro_input(&input).expect("Couldn't parse item");

    let result = builder_for_struct(ast);

    format!("{}", result).parse().expect("Couldn't parse string to tokens")
}

fn filter_attr(attr: &&syn::Attribute) -> bool {
    if attr.style != syn::AttrStyle::Outer {
        return false
    }

    if attr.is_sugared_doc == true {
        if let syn::MetaItem::NameValue(ref ident, _) = attr.value {
            // example:
            // Attribute { style: Outer, value: NameValue(Ident("doc"), Str("/// This is a doc comment for a field", Cooked)), is_sugared_doc: true }
            if ident == "doc" {
                return true
            }
        }
    } else {
        if let syn::MetaItem::List(ref ident, _) = attr.value {
            // example:
            // Attribute { style: Outer, value: List(Ident("allow"), [MetaItem(Word(Ident("non_snake_case")))]), is_sugared_doc: false }
            return match ident.as_ref() {
                "cfg" => true,
                "allow" => true,
                _ => false,
            }
        }
    }
    false
}

fn builder_for_struct(ast: syn::MacroInput) -> quote::Tokens {
    debug!("Deriving Builder for '{}'.", ast.ident);
    let opts = Options::from(ast.attrs);
    if !opts.setter_enabled() {
        trace!("Setters disabled for '{}'.", ast.ident);
        return quote!();
    }
    debug!("Deriving Setters for '{}'.", ast.ident);
    let setter_pattern = opts.setter_pattern();

    let fields = match ast.body {
        syn::Body::Struct(syn::VariantData::Struct(ref fields)) => fields,
        _ => panic!("#[derive(Builder)] can only be used with braced structs"),
    };

    let name = &ast.ident;
    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();
    let funcs = fields.iter().map(|f| {
        let f_name = &f.ident;
        let ty = &f.ty;

        trace!("Filtering field attributes");
        let attrs = f.attrs.iter()
            .filter(|a| {
                let keep = filter_attr(a);
                match keep {
                    true => trace!("Keeping field attribute for setter {:?}", a),
                    false => trace!("Ignoring field attribute {:?}", a)
                }
                keep
            });

        let vis = opts.setter_visibility();
        debug!("Setter visibility = {:?}", vis);

        match *setter_pattern {
            SetterPattern::Owned => quote!(
                    #(#attrs)*
                    #vis fn #f_name<VALUE: Into<#ty>>(self, value: VALUE) -> Self {
                        let mut new = self;
                        new.#f_name = value.into();
                        new
                }),
            SetterPattern::Mutable => quote!(
                    #(#attrs)*
                    #vis fn #f_name<VALUE: Into<#ty>>(&mut self, value: VALUE) -> &mut Self {
                        let mut new = self;
                        new.#f_name = value.into();
                        new
                }),
            SetterPattern::Immutable => quote!(
                    #(#attrs)*
                    #vis fn #f_name<VALUE: Into<#ty>>(&self, value: VALUE) -> Self {
                        let mut new = self.clone();
                        new.#f_name = value.into();
                        new
                }),
        }
    });

    quote! {
        #[allow(dead_code)]
        impl #impl_generics #name #ty_generics #where_clause {
            #(#funcs)*
        }
    }
}
