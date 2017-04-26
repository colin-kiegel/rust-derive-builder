/// Overrides for https://docs.rs/log/#macros
///
/// Source shared by `derive_builder_core` and `derive_builder` via symlink.

macro_rules! log_enabled {
    ($( $x:tt )*) => { false }
}

// delegate to format_args and throw away the result to avoid `unused variable`
// lints.
// The compiler should be able to optimize this away.
macro_rules! debug {
    ($( $x:tt )*) => { format_args!($( $x )*); }
}

macro_rules! error {
    ($( $x:tt )*) => { format_args!($( $x )*); }
}

macro_rules! info {
    ($( $x:tt )*) => { format_args!($( $x )*); }
}

macro_rules! log {
    ($( $x:tt )*) => { format_args!($( $x )*); }
}

macro_rules! trace {
    ($( $x:tt )*) => { format_args!($( $x )*); }
}

macro_rules! warn {
    ($( $x:tt )*) => { format_args!($( $x )*); }
}
