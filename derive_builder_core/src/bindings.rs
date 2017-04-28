use RawTokens;

/// Bindings to be used by the generated code.
#[derive(Debug, Clone, Copy, Default)]
pub struct Bindings {
    /// Whether the generated code should comply with `#![no_std]`.
    pub no_std: bool,
}

impl Bindings {
    /// String type.
    pub fn string_ty(&self) -> RawTokens<&'static str> {
        RawTokens(if self.no_std {
                      ":: collections :: string :: String"
                  } else {
                      ":: std :: string :: String"
                  })
    }

    /// Result type.
    pub fn result_ty(&self) -> RawTokens<&'static str> {
        RawTokens(":: derive_builder :: export :: Result")
    }

    /// Option type.
    pub fn option_ty(&self) -> RawTokens<&'static str> {
        RawTokens(":: derive_builder :: export :: Option")
    }

    /// PhantomData type.
    pub fn phantom_data_ty(&self) -> RawTokens<&'static str> {
        RawTokens(":: derive_builder :: export :: PhantomData")
    }

    /// Default trait.
    pub fn default_trait(&self) -> RawTokens<&'static str> {
        RawTokens(":: derive_builder :: export :: Default")
    }

    /// Clone trait.
    pub fn clone_trait(&self) -> RawTokens<&'static str> {
        RawTokens(":: derive_builder :: export :: Clone")
    }

    /// Into trait.
    pub fn into_trait(&self) -> RawTokens<&'static str> {
        RawTokens(":: derive_builder :: export :: Into")
    }

    /// TryInto trait.
    pub fn try_into_trait(&self) -> RawTokens<&'static str> {
        RawTokens(":: derive_builder :: export :: TryInto")
    }
}

#[test]
fn std() {
    let b = Bindings { no_std: false };

    assert_eq!(b.string_ty().to_tokens(), quote!(::std::string::String));

    assert_eq!(b.result_ty().to_tokens(), quote!(::derive_builder::export::Result));

    assert_eq!(b.option_ty().to_tokens(), quote!(::derive_builder::export::Option));

    assert_eq!(b.phantom_data_ty().to_tokens(),
               quote!(::derive_builder::export::PhantomData));

    assert_eq!(b.default_trait().to_tokens(),
               quote!(::derive_builder::export::Default));

    assert_eq!(b.clone_trait().to_tokens(), quote!(::derive_builder::export::Clone));

    assert_eq!(b.into_trait().to_tokens(), quote!(::derive_builder::export::Into));
}

#[test]
fn no_std() {
    let b = Bindings { no_std: true };

    assert_eq!(b.string_ty().to_tokens(),
               quote!(::collections::string::String));

    assert_eq!(b.result_ty().to_tokens(), quote!(::derive_builder::export::Result));

    assert_eq!(b.option_ty().to_tokens(), quote!(::derive_builder::export::Option));

    assert_eq!(b.phantom_data_ty().to_tokens(),
               quote!(::derive_builder::export::PhantomData));

    assert_eq!(b.default_trait().to_tokens(),
               quote!(::derive_builder::export::Default));

    assert_eq!(b.clone_trait().to_tokens(), quote!(::derive_builder::export::Clone));

    assert_eq!(b.into_trait().to_tokens(), quote!(::derive_builder::export::Into));
}
