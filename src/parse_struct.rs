// from <https://github.com/diesel-rs/diesel/blob/8bebe479cb05388220719b8b385fffb727d40588/diesel/src/macros/parse.rs>
// Copyright (C) 2016 Sean Griffin, licensed under MIT/Apache-2.0

#[macro_export]
#[doc(hidden)]
macro_rules! __parse_struct_body {
    // Entry point for named structs
    (
        $headers:tt,
        callback = $callback:ident,
        body = {$($body:tt)*},
    ) => {
        __parse_struct_body! {
            $headers,
            callback = $callback,
            body = ($($body)*,),
            buff = ( attr [] ),
            fields = [],
        }
    };

    // Entry point for tuple structs
    (
        $headers:tt,
        callback = $callback:ident,
        body = ($($body:tt)*),
    ) => {
        __parse_struct_body! {
            $headers,
            callback = $callback,
            body = ($($body)*,),
            buff = ( attr [] ),
            fields = [],
        }
    };

    // parse attributes for fields (e.g. doc-comments)
    (
        $headers:tt,
        callback = $callback:ident,
        body = (
            #[$meta:meta]
            $($tail:tt)*),
        buff = ( attr [$($attr:tt)*] ),
        fields = $fields:tt,
    ) => {
        __parse_struct_body! {
            $headers,
            callback = $callback,
            body = ($($tail)*),
            buff = ( attr [$($attr:tt)* #[$meta]] ),
            fields = $fields,
        }
    };

    // silently skip visibility specifier
    (
        $headers:tt,
        callback = $callback:ident,
        body = (
            pub
            $($tail:tt)*),
        buff = $buff:tt,
        fields = $fields:tt,
    ) => {
        __parse_struct_body! {
            $headers,
            callback = $callback,
            body = ($($tail)*),
            buff = $buff,
            fields = $fields,
        }
    };

    // Since we blindly add a comma to the end of the body, we might have a
    // double trailing comma.  If it's the only token left, that's what
    // happened. Strip it.
    (
        $headers:tt,
        callback = $callback:ident,
        body = (,),
        buff = $buff:tt,
        fields = $fields:tt,
    ) => {
        __parse_struct_body! {
            $headers,
            callback = $callback,
            body = (),
            buff = $buff,
            fields = $fields,
        }
    };

    // handle struct field and its type
    (
        $headers:tt,
        callback = $callback:ident,
        body = ($field_name:ident : $field_ty:ty, $($tail:tt)*),
        buff = ( attr $field_attr:tt ),
        fields = [$($fields:tt)*],
    ) => {
        __parse_struct_body! {
            $headers,
            callback = $callback,
            body = ($($tail)*),
            buff = ( attr [] ),
            fields = [$($fields)* {
                field_name: $field_name,
                field_ty: $field_ty,
                field_attr: $field_attr,
            }],
        }
    };

    // At this point we've parsed the entire body. We create the pattern
    // for destructuring, and pass all the information back to the main macro
    // to generate the final impl
    (
        $headers:tt,
        callback = $callback:ident,
        body = (),
        buff = ( attr [] ),
        fields = $fields:tt,
    ) => {
        $callback! {
            $headers,
            fields = $fields,
        }
    };
}

/// Hack to tell the compiler that something is in fact an item. This is needed
/// when `tt` fragments are used in specific positions.
#[doc(hidden)]
#[macro_export]
macro_rules!  __parse_as_item {
    ($i:item) => { $i }
}
