/// `Builder!` macro
///
/// Use in combiantion with [custom_derive][custom_derive].
///
/// [custom_derive]: https://crates.io/crates/custom_derive
#[macro_export]
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
        Builder! { $($body)* }
    };

    // Struct with generics.
    //
    // Receive parsed fields of normal struct from `__diesel_parse_struct_body`
    // and add implementation.
    //
    // These patterns must appear above those which start with an ident to
    // compile.
    (
        (
            struct_name = $struct_name:ident,
            generics = ($($generics:ident),*),
        ),
        fields = [$({
            field_name: $field_name:ident,
            column_name: $column_name:ident,
            field_ty: $field_ty:ty,
            field_kind: $field_kind:ident,
        })+],
    ) => {
        #[allow(dead_code)]
        impl<$($generics),*> $struct_name<$($generics),*> {
            $(
                fn $field_name<VALUE: Into<$field_ty>>(mut self, value: VALUE) -> Self {
                    self.$field_name = value.into();
                    self
                }
            )+
        }
    };

    // Struct without generics.
    //
    // Receive parsed fields of normal struct from `__diesel_parse_struct_body`
    // and add implementation.
    //
    // These patterns must appear above those which start with an ident to
    // compile.
    (
        (
            struct_name = $struct_name:ident,
            generics = (),
        ),
        fields = [$({
            field_name: $field_name:ident,
            column_name: $column_name:ident,
            field_ty: $field_ty:ty,
            field_kind: $field_kind:ident,
        })+],
    ) => {
        #[allow(dead_code)]
        impl $struct_name {
            $(
                fn $field_name<VALUE: Into<$field_ty>>(mut self, value: VALUE) -> Self {
                    self.$field_name = value.into();
                    self
                }
            )+
        }
    };

    // Handle struct with generics
    (
        $struct_name:ident <$($generics:ident),*>
        $body:tt $(;)*
    ) => {
        __diesel_parse_struct_body! {
            (
                struct_name = $struct_name,
                generics = ($($generics),*),
            ),
            callback = Builder,
            body = $body,
        }
    };

    // Handle struct with no generics
    (
        $struct_name:ident
        $body:tt $(;)*
    ) => {
        __diesel_parse_struct_body! {
            (
                struct_name = $struct_name,
                generics = (),
            ),
            callback = Builder,
            body = $body,
        }
    };
}
