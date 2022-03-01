use darling::{self, FromMeta};

/// Controls the signature of a setter method,
/// more specifically how `self` is passed and returned.
///
/// It can also be generalized to methods with different parameter sets and
/// return types, e.g. the `build()` method.
#[derive(PartialEq, Eq, Debug, Clone, Copy, FromMeta)]
pub enum BuilderPattern {
    /// E.g. `fn bar(self, bar: Bar) -> Self`.
    Owned,
    /// E.g. `fn bar(&mut self, bar: Bar) -> &mut Self`.
    Mutable,
    /// E.g. `fn bar(&self, bar: Bar) -> Self`.
    ///
    /// Note:
    /// - Needs to `clone` in order to return an _updated_ instance of `Self`.
    /// - There is a great chance that the Rust compiler (LLVM) will
    ///   optimize chained `clone` calls away in release mode.
    ///   Therefore this turns out not to be as bad as it sounds.
    Immutable,
}

impl BuilderPattern {
    /// Returns true if this style of builder needs to be able to clone its
    /// fields during the `build` method.
    pub fn requires_clone(&self) -> bool {
        *self != Self::Owned
    }
}

/// Defaults to `Mutable`.
impl Default for BuilderPattern {
    fn default() -> Self {
        Self::Mutable
    }
}

#[derive(Debug, Clone)]
pub struct Each {
    pub name: syn::Ident,
    pub into: bool,
}

/// Create `Each` from an attribute's `Meta`.
///
/// Two formats are supported:
///
/// * `each = "..."`, which provides the name of the `each` setter and otherwise uses default values
/// * `each(name = "...")`, which allows setting additional options on the `each` setter
impl FromMeta for Each {
    fn from_value(value: &syn::Lit) -> darling::Result<Self> {
        if let syn::Lit::Str(v) = value {
            Ok(Self {
                name: v.parse()?,
                into: false,
            })
        } else {
            Err(darling::Error::unexpected_lit_type(value))
        }
    }

    fn from_list(items: &[syn::NestedMeta]) -> darling::Result<Self> {
        #[derive(FromMeta)]
        struct EachOpts {
            name: syn::Ident,
            #[darling(default)]
            into: bool,
        }

        impl From<EachOpts> for Each {
            fn from(v: EachOpts) -> Each {
                // Destructure `v` without using `..` to make sure every field is read...
                let EachOpts { name, into } = v;

                // ... and create `Self` without using `..` to ensure every field is propagated
                Self { name, into }
            }
        }

        EachOpts::from_list(items).map(Each::from)
    }
}
