use quote::Tokens;

/// Controls the signature of a setter method,
/// more specifically how `self` is passed and returned.
///
/// It can also be generalized to methods with different parameter sets and return types,
/// e.g. the `build()` method.
#[derive(PartialEq, Debug, Clone, Copy)]
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
    /// Expresses this pattern
    pub fn to_build_method_tokens(&self) -> Tokens {
        match *self {
            BuilderPattern::Owned => quote!(self),
            BuilderPattern::Mutable |
            BuilderPattern::Immutable => quote!(&self),
        }
    }
}

/// Defaults to `Mutable`.
impl Default for BuilderPattern {
    fn default() -> BuilderPattern {
        BuilderPattern::Mutable
    }
}