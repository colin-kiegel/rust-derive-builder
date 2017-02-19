//! Derive a builder for a struct
//!
//! This crate implements the [builder pattern] for you.
//! Just apply `#[derive(Builder)]` to a struct `Foo`, and it will derive an additional
//! struct `FooBuilder` with **setter**-methods for all fields and a **build**-method
//! — the way you want it.
//!
//! # Quick Start
//!
//! Add `derive_builder` as a dependency to you `Cargo.toml`.
//!
//! ## What you write
//!
//! ```rust
//! #[macro_use]
//! extern crate derive_builder;
//!
//! #[derive(Builder)]
//! struct Lorem {
//!     ipsum: String,
//!     // ..
//! }
//! # fn main() {}
//! ```
//!
//! ## What you get
//!
//! ```rust
//! # #[macro_use]
//! # extern crate derive_builder;
//! #
//! # struct Lorem {
//! #     ipsum: String,
//! # }
//! # fn main() {}
//!
//! #[derive(Clone, Default)]
//! struct LoremBuilder {
//!     ipsum: Option<String>,
//! }
//!
//! #[allow(dead_code)]
//! impl LoremBuilder {
//!     pub fn ipsum<VALUE: Into<String>>(&mut self, value: VALUE) -> &mut Self {
//!         let mut new = self;
//!         new.ipsum = Some(value.into());
//!         new
//!     }
//!     fn build(&self) -> Result<Lorem, String> {
//!         Ok(Lorem {
//!             ipsum: Clone::clone(self.ipsum
//!                 .as_ref()
//!                 .ok_or("ipsum must be initialized")?),
//!         })
//!     }
//! }
//! ```
//!
//! By default all generated setter-methods take and return `&mut self`
//! (aka _non-conusuming_ builder pattern). Accordingly the build method also takes a
//! reference by default.
//!
//! You can easily opt into different patterns and control many other aspects.
//!
//! The build method returns `Result<T, String>`, where `T` is the struct you started with.
//! It returns `Err<String>` if you didn't initialize all fields.
//!
//! # Builder Patterns
//!
//! Let's look again at the example above. You can build structs like this:
//!
//! ```rust
//! # #[macro_use] extern crate derive_builder;
//! # #[derive(Builder)] struct Lorem { ipsum: String }
//! # fn try_main() -> Result<(), String> {
//! let x: Lorem = LoremBuilder::default().ipsum("42").build()?;
//! # Ok(())
//! # } fn main() { try_main().unwrap(); }
//! ```
//!
//! Ok, _chaining_ method calls is nice, but what if `ipsum("42")` should only happen if `geek = true`?
//!
//! So let's make this call conditional
//!
//! ```rust
//! # #[macro_use] extern crate derive_builder;
//! # #[derive(Builder)] struct Lorem { ipsum: String }
//! # fn try_main() -> Result<(), String> {
//! # let geek = true;
//! let mut builder = LoremBuilder::default();
//! if geek {
//!     builder.ipsum("42");
//! }
//! let x: Lorem = builder.build()?;
//! # Ok(())
//! # } fn main() { try_main().unwrap(); }
//! ```
//!
//! Now it comes in handy that our setter methods takes and returns a mutable reference. Otherwise
//! we would need to write something more clumsy like `builder = builder.ipsum("42")` to reassign
//! the return value each time we have to call a setter conditionally.
//!
//! Setters with mutable references are therefore a convenient default for the builder
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
//! You can precede your struct (or field) with `#[builder(pattern="mutable")]`
//! to make this choice explicit.
//!
//! * Setters take and return `&mut self`.
//! * PRO: Setter calls and final build method can be chained.
//! * CON: The build method must clone or copy data to create something owned out of a
//!   mutable reference. Otherwise it could not be used in a chain. **(*)**
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
//! ## Generic structs
//!
//! ```rust
//! #[macro_use]
//! extern crate derive_builder;
//!
//! #[derive(Builder, Debug, PartialEq, Default, Clone)]
//! struct GenLorem<T: Clone> {
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
//! to the according builder fields and setter-methods, if it is one of the following:
//!
//! * `/// ...`
//! * `#[doc = ...]`
//! * `#[cfg(...)]`
//! * `#[allow(...)]`
//!
//! The whitelisting minimizes interference with other custom attributes like Serde/Diesel etc.
//!
//! ```rust
//! #[macro_use]
//! extern crate derive_builder;
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
//! Otherwise precede your struct (or field) with `#[builder(private)]` to opt into private
//! setters.
//!
//! ## Setter Prefixes
//!
//! Setter methods are named after their corresponding field by default.
//!
//! You can precede your struct (or field) with e.g. `#[builder(setter_prefix="xyz")` to change
//! the method name to `xyz_foo` if the field is named `foo`. Note that an underscore is included
//! by default, since Rust favors snake case here.
//!
//! # Troubleshooting
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
//! ## Report Issues and Ideas
//!
//! https://github.com/colin-kiegel/rust-derive-builder/issues
//!
//! If possible please try to provide the debugging info if you experience unexpected
//! compilation errors (see above).
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
use options::{StructOptions, FieldOptions, OptionsBuilder, FieldMode, SetterPattern};
use std::sync::atomic::{AtomicBool, Ordering, ATOMIC_BOOL_INIT};

type TokenVec = Vec<quote::Tokens>;
type AttrVec<'a> = Vec<&'a syn::Attribute>;

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

    result.parse().expect(&format!("Couldn't parse `{}` to tokens", result))
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
    debug!("Deriving Builder for `{}`.", ast.ident);
    let opts = StructOptions::from(&ast);

    let fields = match ast.body {
        syn::Body::Struct(syn::VariantData::Struct(ref fields)) => fields,
        _ => panic!("#[derive(Builder)] can only be used with braced structs"),
    };

    let struct_name = &ast.ident;
    let builder_name = syn::Ident::from(opts.builder_name());
    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();

    let mut setter_fns     = TokenVec::with_capacity(fields.len());
    let mut builder_fields = TokenVec::with_capacity(fields.len());
    let mut initializers   = TokenVec::with_capacity(fields.len());

    for f in fields {
        let name = f.ident.as_ref()
            .expect(&format!("Missing identifier for field `{:?}`.", f))
            .as_ref();
        trace!("Parsing field `{}`.", name);

        let f_opts = OptionsBuilder::<FieldMode>::default()
            .parse_attributes(&f.attrs)
            .build(name, &opts);

        trace!("Filtering field attributes");
        let attrs: AttrVec = f.attrs.iter()
            .filter(|a| {
                let keep = filter_attr(a);
                match keep {
                    true => trace!("Keeping field attribute for builder field and setter `{:?}`", a),
                    false => trace!("Ignoring field attribute for builder field and setter `{:?}`", a)
                }
                keep
            }).collect();

        setter_fns.push(derive_setter(f, &f_opts, &attrs));
        builder_fields.push(derive_builder_field(f, &f_opts, &attrs));
        initializers.push(derive_initializer(f, &f_opts));
        // NOTE: Looking forward for computation in interpolation
        // - https://github.com/dtolnay/quote/issues/10
        // => `quote!(#{f.vis} ...)
    }

    let builder_vis = opts.builder_visibility();
    let build_fn = {
        let ref_self = match *opts.field_defaults().setter_pattern() {
            SetterPattern::Owned => quote!(self),
            SetterPattern::Mutable => quote!(&self),
            SetterPattern::Immutable => quote!(&self),
        };
        quote!(
            #builder_vis fn build(#ref_self) -> ::std::result::Result<#struct_name #ty_generics, ::std::string::String> {
                Ok(#struct_name {
                    #(#initializers)*
                })
            }
        )
    };

    // We need to `#[derive(Clone)]` only for the immutable builder pattern
    quote! {
        #[derive(Default, Clone)]
        #builder_vis struct #builder_name #ty_generics #where_clause {
            #(#builder_fields)*
        }

        #[allow(dead_code)]
        impl #impl_generics #builder_name #ty_generics #where_clause {
            #(#setter_fns)*

            #build_fn
        }
    }
}

fn derive_builder_field(f: &syn::Field, opts: &FieldOptions, attrs: &AttrVec)
    -> quote::Tokens
{
    if opts.setter_enabled() {
        trace!("Deriving builder field for `{}``.", opts.field_name());
        let (vis, ident, ty) = (&f.vis, &f.ident, &f.ty);
        quote!(#(#attrs)* #vis #ident: ::std::option::Option<#ty>,)
    } else {
        trace!("Skipping builder field for `{}`.", opts.field_name());
        quote!()
    }
}

fn derive_initializer(f: &syn::Field, opts: &FieldOptions) -> quote::Tokens {
    trace!("Deriving initializer for `{}`.", opts.field_name());

    let err_uninitizalied = format!("`{}` must be initialized", opts.field_name());
    let pattern = opts.setter_pattern();
    let ident = &f.ident;

    if opts.setter_enabled() {
        let initializer = match *pattern {
            SetterPattern::Owned => quote!(
                    #ident: self.#ident.ok_or(#err_uninitizalied)?,
                ),
            SetterPattern::Mutable |
            SetterPattern::Immutable => quote!(
                    #ident: ::std::clone::Clone::clone(self.#ident.as_ref().ok_or(#err_uninitizalied)?),
                ),
        };

        debug!("Initializer is `{:?}`", initializer);

        initializer
    } else {
        trace!("Fallback to default initializer for `{}`.", opts.field_name());
        quote!( #ident: default::Default(), )
    }
}

fn derive_setter(f: &syn::Field, opts: &FieldOptions, attrs: &AttrVec)
    -> quote::Tokens
{
    if opts.setter_enabled() {
        trace!("Deriving setter for `{}`.", opts.field_name());
        let ty = &f.ty;
        let pattern = opts.setter_pattern();
        let vis = opts.setter_visibility();
        let fieldname = f.ident.as_ref()
            .expect(&format!("Missing identifier for field `{:?}`.", f));
        let funcname = if opts.setter_prefix().len() > 0 {
            Cow::Owned(syn::Ident::new(format!("{}_{}", opts.setter_prefix(), fieldname)))
        } else {
            Cow::Borrowed(fieldname)
        };

        let setter = match *pattern {
            SetterPattern::Owned => quote!(
                    #(#attrs)*
                    #vis fn #funcname<VALUE: ::std::convert::Into<#ty>>(self, value: VALUE) -> Self {
                        let mut new = self;
                        new.#fieldname = ::std::option::Option::Some(value.into());
                        new
                }),
            SetterPattern::Mutable => quote!(
                    #(#attrs)*
                    #vis fn #funcname<VALUE: ::std::convert::Into<#ty>>(&mut self, value: VALUE) -> &mut Self {
                        let mut new = self;
                        new.#fieldname = ::std::option::Option::Some(value.into());
                        new
                }),
            SetterPattern::Immutable => quote!(
                    #(#attrs)*
                    #vis fn #funcname<VALUE: ::std::convert::Into<#ty>>(&self, value: VALUE) -> Self {
                        let mut new = ::std::clone::Clone::clone(self);
                        new.#fieldname = ::std::option::Option::Some(value.into());
                        new
                }),
        };

        debug!("Setter is `{:?}`", setter);

        setter
    } else {
        trace!("Skipping setter for `{}`.", opts.field_name());
        quote!()
    }
}
