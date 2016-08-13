/// `Builder!` macro
///
/// Only useful in combination with [custom_derive][custom_derive].
///
/// [custom_derive]: https://crates.io/crates/custom_derive
#[macro_export]
#[doc(hidden)]
macro_rules! Builder {
    // Strip empty argument list if given (Passed by custom_derive macro)
    (() $($body:tt)*) => {
        Builder! { $($body)* }
    };

    // Strip meta items, pub (if present) and struct from definition
    (
        $(#[$ignore:meta])*
        $(pub)* struct $($body:tt)*
    ) => {
        Builder! { @parse $($body)* }
    };

    // Struct with + without generics.
    //
    // Receive parsed fields of normal struct from `__parse_struct_body`
    // and add implementation.
    //
    // These patterns must appear above those which start with an ident to
    // compile.
    (
        @impl_struct
        struct_name = $struct_name:ident,
        generics = ($($generics:ident),*),
        fields = [$({
            field_name: $field_name:ident,
            field_ty: $field_ty:ty,
            field_attr: [$($attr:tt)*],
        })*],
    ) => {
        #[allow(dead_code)]
        impl<$($generics),*> $struct_name<$($generics),*> {
            $(
                Builder!( @impl_setter {
                    field_attr_raw: [$($attr)*],
                    field_attr_filtered: [],
                    field_name: $field_name,
                    field_ty: $field_ty,
                });
            )*
        }
    };

    // Handle struct with generics
    (
        @parse
        $struct_name:ident <$($generics:ident),*>
        $body:tt $(;)*
    ) => {
        __parse_struct_body! {
            @parse
            body = $body,
            callback = Builder (
                @impl_struct
                struct_name = $struct_name,
                generics = ($($generics),*),
            )
        }
    };

    // Handle struct with no generics
    (
        @parse
        $struct_name:ident
        $body:tt $(;)*
    ) => {
        __parse_struct_body! {
            @parse
            body = $body,
            callback = Builder (
                @impl_struct
                struct_name = $struct_name,
                generics = (),
            )
        }
    };

    // implement setter when all raw attributes have been filtered
    ( @impl_setter {
        field_attr_raw: [],
        field_attr_filtered: [$(#[$meta:meta])*],
        field_name: $field_name:ident,
        field_ty: $field_ty:ty,
    }) => {
        $(#[$meta])*
        pub fn $field_name<VALUE: Into<$field_ty>>(&mut self, value: VALUE) -> &mut Self {
            self.$field_name = value.into();
            self
        }
    };

    // allow attribute: #[doc = ... ]
    ( @impl_setter {
        field_attr_raw: [#[doc = $($doc:tt)*] $($attr_raw:tt)*],
        field_attr_filtered: [$($attr_filtered:tt)*],
        $($tail:tt)*
    }) => {
        Builder!( @impl_setter {
            field_attr_raw: [$($attr_raw)*],
            field_attr_filtered: [#[doc = $($doc)*] $($attr_filtered)*],
            $($tail)*
        });
    };

    // allow attribute: #[allow (...)]
    ( @impl_setter {
        field_attr_raw: [#[allow($($allow:tt)*)] $($attr_raw:tt)*],
        field_attr_filtered: [$($attr_filtered:tt)*],
        $($tail:tt)*
    }) => {
        Builder!( @impl_setter {
            field_attr_raw: [$($attr_raw)*],
            field_attr_filtered: [#[allow($($allow)*)] $($attr_filtered)*],
            $($tail)*
        });
    };

    // allow attribute: #[cfg (...)]
    ( @impl_setter {
        field_attr_raw: [#[cfg($($cfg:tt)*)] $($attr_raw:tt)*],
        field_attr_filtered: [$($attr_filtered:tt)*],
        $($tail:tt)*
    }) => {
        Builder!( @impl_setter {
            field_attr_raw: [$($attr_raw)*],
            field_attr_filtered: [#[cfg($($cfg)*)] $($attr_filtered)*],
            $($tail)*
        });
    };

    // ignore attributes, that have not been whitelisted
    ( @impl_setter {
        field_attr_raw: [#[$($ignore:tt)*] $($attr_raw:tt)*],
        field_attr_filtered: [$($attr_filtered:tt)*],
        $($tail:tt)*
    }) => {
        Builder!( @impl_setter {
            field_attr_raw: [$($attr_raw)*],
            field_attr_filtered: [$($attr_filtered)*],
            $($tail)*
        });
    };
}
