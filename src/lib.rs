//! Derive a builder for a struct
//!
//! This crate implements the [builder pattern].
//! When applied to a struct, it will derive **setter-methods** for all struct fields
//! — the way you want it.
//!
//! # Quick Start
//!
//! ## Generate Setters
//!
//! ```rust
//! #[macro_use] extern crate derive_builder;
//!
//! #[derive(Builder)]
//! struct Lorem {
//!     ipsum: String,
//!     // ..
//! }
//! # fn main() {}
//! ```
//!
//! `#[derive(Builder)]` will automatically generate a setter method for the `ipsum` field,
//! looking like this:
//!
//! ```rust,ignore
//! pub fn ipsum<VALUE: Into<String>>(&mut self, value: VALUE) -> &mut Self {
//!     self.ipsum = value.into();
//!     self
//! }
//! ```
//!
//! By default all generated setter-methods take and return `&mut self`
//! (aka _non-conusuming_ builder pattern). Don't worry, you can easily opt into different
//! patterns and control many other aspects.
//!
//! ## Add a Build Method
//!
//! Ok, we've got setters. To complete the builder pattern you only have to implement at least
//! one method which actually builds something based on the struct.
//!
//! These custom build methods of yours should also take `&mut self`, if you stick with the
//! non-consuming pattern.
//!
//! This could look like:
//!
//! ```rust
//! #[macro_use] extern crate derive_builder;
//!
//! #[derive(Builder, Default)]
//! struct Lorem {
//!     pub ipsum: String,
//!     // ..
//! }
//!
//! fn main() {
//!     let x = LoremBuilder::default().ipsum("42").build().unwrap();
//!     println!("{:?}", x.ipsum);
//! }
//! ```
//!
//! # Builder Patterns
//!
//! Let's look again at `let x = Lorem::default().ipsum("42").build()`.
//! Chaining method calls is nice, but what if `ipsum("42")` should only happen if `geek = true`?
//!
//! So let's make this call conditional
//!
//! ```rust,ignore
//! let mut builder = Lorem::default();
//! if geek {
//!     builder.ipsum("42");
//! }
//! let x = builder.build();
//! ```
//!
//! Now it comes in handy that our setter methods takes and returns a mutable reference. Otherwise
//! we would need to write something more clumsy like `builder = builder.ipsum("42")` to reassign
//! the return value each time we have to call a setter conditionally.
//!
//! Setters with mutable references are therefore the recommended choice for the builder
//! pattern in Rust.
//!
//! But this is a free world and the choice is still yours.
//!
//! ## Owned, aka Consuming
//!
//! Precede your struct (or field) with `#[builder(pattern="owned")]` to opt into this pattern.
//!
//! * Setters take and return `self`.
//! * PRO: Setter calls and final build method can be chained.
//! * CON: If you don't chain your calls, you have to create a reference to each return value,
//!   e.g. `builder = builder.ipsum("42")`.
//!
//! ## Mutable, aka Non-Comsuming (recommended)
//!
//! This pattern is recommended and active by default if you don't specify anything else.
//! You can precede your struct (or field) with `#[builder(pattern="mutable")]` to make this choice explicit.
//!
//! * Setters take and return `&mut self`.
//! * PRO: Setter calls and final build method can be chained.
//! * CON: The build method must clone or copy data to create something owned out of a
//!   mutable reference. Otherwise it can not be used in a chain. **(*)**
//!
//! ## Immutable
//!
//! Precede your struct (or field) with `#[builder(pattern="immutable")]` to opt into this pattern.
//!
//! * Setters take and return `&self`.
//! * PRO: Setter calls and final build method can be chained.
//! * CON: If you don't chain your calls, you have to create a reference to each return value,
//!   e.g. `builder = builder.ipsum("42")`.
//! * CON: The build method _and each setter_ must clone or copy data to create something owned
//!   out of a reference. **(*)**
//!
//! ## (*) Performance Considerations
//!
//! Luckily Rust is clever enough to optimize these clone-calls away in release builds
//! for your every-day use cases. Thats quite a safe bet - we checked this for you. ;-)
//! Switching to consuming signatures (=`self`) is unlikely to give you any performance
//! gain, but very likely to restrict your API for non-chained use cases.
//!
//! # More Features
//!
//! We'll pretend that `clone()` is our build method for the following examples, to keep them as
//! short as possible.
//!
//! ## Generic structs
//!
//! ```rust
//! #[macro_use] extern crate derive_builder;
//!
//! #[derive(Builder, Debug, PartialEq, Default, Clone)]
//! struct GenLorem<T> where
//!     T: Clone
//! {
//!     ipsum: String,
//!     dolor: T,
//! }
//!
//! fn main() {
//!     let x = GenLoremBuilder::default().ipsum("sit").dolor(42).build().unwrap();
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
//! ## Setter Visibility
//!
//! Setters are public by default. You can precede your struct (or field) with `#[builder(public)]`
//! to make this explicit.
//!
//! Otherwise precede your struct (or field) with `#[builder(private)]` to opt into private setters.
//!
//! ## Setter Prefixes
//!
//! Setter methods are named after their corresponding field by default.
//!
//! You can precede your struct (or field) with e.g. `#[builder(setter_prefix="xyz")` to change the method
//! name to `xyz_foo` if the field is named `foo`. Note that an underscore is included by default,
//! since Rust favors snake case here.
//!
//! ## Gotchas
//!
//! - Tuple structs and unit structs are not supported as they have no field
//!   names.
//! - When defining a generic struct, you cannot use `VALUE` as a generic
//!   parameter as this is what all setters are using.
//!
//! ## Debugging Info
//!
//! If you experience any problems during compilation, you can enable additional debug output
//! by setting the environment variable `RUST_LOG=derive_builder=trace` before you call `cargo`
//! or `rustc`. Example: `RUST_LOG=derive_builder=trace cargo test`.
//!
//! [builder pattern]: https://aturon.github.io/ownership/builders.html

#![crate_type = "proc-macro"]

extern crate proc_macro;
extern crate syn;
#[macro_use]
extern crate quote;
#[macro_use]
extern crate log;
extern crate env_logger;

mod options;

use std::borrow::Cow;
use proc_macro::TokenStream;
use quote::ToTokens;
use options::{StructOptions, FieldOptions, OptionsBuilder, FieldMode, SetterPattern};
use std::sync::atomic::{AtomicBool, Ordering, ATOMIC_BOOL_INIT};

// beware: static muts are not threadsafe. :-)
static mut LOGGER_INITIALIZED: AtomicBool = ATOMIC_BOOL_INIT; // false

#[doc(hidden)]
#[proc_macro_derive(Builder, attributes(builder))]
pub fn derive(input: TokenStream) -> TokenStream {
    if unsafe { !LOGGER_INITIALIZED.compare_and_swap(false, true, Ordering::SeqCst) } {
        env_logger::init().unwrap();
    }

    let input = input.to_string();

    let ast = syn::parse_macro_input(&input).expect("Couldn't parse item");

    let result = builder_for_struct(ast).to_string();

    result.parse().expect(&format!("Couldn't parse {:?} to tokens", result))
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
    let opts = StructOptions::from(&ast);

    let fields = match ast.body {
        syn::Body::Struct(syn::VariantData::Struct(ref fields)) => fields,
        _ => panic!("#[derive(Builder)] can only be used with braced structs"),
    };

    let struct_name = &ast.ident;
    let builder_name = syn::Ident::from(opts.builder_name());
    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();

    let mut funcs          = Vec::<quote::Tokens>::with_capacity(fields.len());
    let mut builder_fields = Vec::<quote::Tokens>::with_capacity(fields.len());
    let mut initializers   = Vec::<quote::Tokens>::with_capacity(fields.len());

    for f in fields {
        let ident = &f.ident;
        let vis = &f.vis;
        let ty = &f.ty;
        // let attrs = &f.attrs;
        let name = f.ident.as_ref()
            .expect(&format!("Missing identifier for field {:?}.", f))
            .as_ref();
        trace!("Parsing field {}.", name);

        let f_opts = OptionsBuilder::<FieldMode>::default()
            .parse_attributes(&f.attrs)
            .build(name, &opts);

        // Setter
        if f_opts.setter_enabled() {
            let mut tokens = quote::Tokens::new();
            derive_setter(f, &f_opts).to_tokens(&mut tokens);
            funcs.push(tokens);
        } else {
            trace!("Skipping setter for {}.", name);
        }

        // Initializer
        if f_opts.setter_enabled() {
            // let mut tokens = quote::Tokens::new();
            // derive_initializer(f, &f_opts).to_tokens(&mut tokens);
            initializers.push(derive_initializer(f, &f_opts));
        } else {
            trace!("Fallback to default initializer for {}.", name);
            initializers.push(quote!( #ident: default::Default(), ));
        }

        // builder fields
        if f_opts.setter_enabled() {
            builder_fields.push(quote!(#vis #ident: Option<#ty>,));
        } else {
            trace!("Skipping builder field for {}.", name);
        };
        // NOTE: Looking forward for computation in interpolation
        // - https://github.com/dtolnay/quote/issues/10
        // => `quote!(#{f.vis} ...)
    }

    let ref_self = match *opts.field_defaults().setter_pattern() {
        SetterPattern::Owned => quote!(self),
        SetterPattern::Mutable => quote!(&self),
        SetterPattern::Immutable => quote!(&self),
    };
    debug!("{:?}", ref_self);
    let build = quote!(
        fn build(#ref_self) -> Result<#struct_name #ty_generics, String> {
            Ok(#struct_name{
                #(#initializers)*
            })
        }
    );

    let builder_vis = opts.builder_visibility();

    // We need to `#[derive(Clone)]` only for the immutable builder pattern
    quote! {
        #[derive(Default, Clone)]
        #builder_vis struct #builder_name #ty_generics #where_clause {
            #(#builder_fields)*
        }

        #[allow(dead_code)]
        impl #impl_generics #builder_name #ty_generics #where_clause {
            #(#funcs)*

            #builder_vis #build
        }
    }
}

fn derive_initializer(f: &syn::Field, opts: &FieldOptions) -> quote::Tokens {
    trace!("Deriving initializer for {}.", opts.field_name());

    let err_uninitizalied = format!("{} must be initialized", opts.field_name());
    let pattern = opts.setter_pattern();
    let ident = &f.ident;
    // let attrs = &f.attrs;

    let initializer = match *pattern {
        SetterPattern::Owned => quote!(
                #ident: self.#ident.ok_or(#err_uninitizalied)?,
            ),
        SetterPattern::Mutable
        | SetterPattern::Immutable => quote!(
                #ident: Clone::clone(self.#ident.as_ref().ok_or(#err_uninitizalied)?),
            ),
    };

    debug!("Initializer is {:?}", initializer);

    initializer
}

fn derive_setter(f: &syn::Field, opts: &FieldOptions) -> quote::Tokens {
    trace!("Deriving setter for {:?}.", opts.field_name());
    let ty = &f.ty;
    let pattern = opts.setter_pattern();
    let vis = opts.setter_visibility();
    let fieldname = f.ident.as_ref().expect(&format!("Missing identifier for field {:?}.", f));
    let funcname = if opts.setter_prefix().len() > 0 {
        Cow::Owned(syn::Ident::new(format!("{}_{}", opts.setter_prefix(), fieldname)))
    } else {
        Cow::Borrowed(fieldname)
    };

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

    let setter = match *pattern {
        SetterPattern::Owned => quote!(
                #(#attrs)*
                #vis fn #funcname<VALUE: Into<#ty>>(self, value: VALUE) -> Self {
                    let mut new = self;
                    new.#fieldname = Some(value.into());
                    new
            }),
        SetterPattern::Mutable => quote!(
                #(#attrs)*
                #vis fn #funcname<VALUE: Into<#ty>>(&mut self, value: VALUE) -> &mut Self {
                    let mut new = self;
                    new.#fieldname = Some(value.into());
                    new
            }),
        SetterPattern::Immutable => quote!(
                #(#attrs)*
                #vis fn #funcname<VALUE: Into<#ty>>(&self, value: VALUE) -> Self {
                    let mut new = Clone::clone(self);
                    new.#fieldname = Some(value.into());
                    new
            }),
    };

    debug!("Setter is {:?}", setter);

    setter
}
