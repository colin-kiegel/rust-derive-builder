/// Helper macro to generate a setter for some `Option<T>`.
///
/// The setter will panic if the Option is already initialized.
///
/// # Examples
///
/// ```rust, ignore
/// OptionsBuilder {
///    foo: Option<u32>,
/// }
///
/// impl OptionsBuilder {
///     impl_setter!{
///        ident: foo,
///        desc: "foo",
///        map: |x: u32| { x },
///    }
/// }
/// ```
macro_rules! impl_setter {
    (
        ident: $ident:ident,
        desc: $desc:expr,
        map: |$x:ident: $ty:ty| {$( $map:tt )*},
    ) => {
        impl_setter!{
            ident: $ident for $ident,
            desc: $desc,
            map: |$x: $ty| {$( $map )*},
        }
    };
    (
        ident: $setter:ident for $field:ident,
        desc: $desc:expr,
        map: |$x:ident: $ty:ty| {$( $map:tt )*},
    ) => {
        fn $setter(&mut self, $x: $ty) {
            if let Some(ref current) = self.$field {
                panic!("Failed to set {} to `{:?}` (already defined as `{:?}`) {}.",
                    $desc,
                    $x,
                    current,
                    self.where_diagnostics());
            }
            self.$field = Some({$( $map )*});
        }
    }
}
