// from <https://github.com/diesel-rs/diesel/blob/8bebe479cb05388220719b8b385fffb727d40588/diesel/src/macros/parse.rs>
// Copyright (C) 2016 Sean Griffin, licensed under MIT/Apache-2.0

#[macro_export]
#[doc(hidden)]
macro_rules! __parse_struct_body {
    // Entry point for named structs
    (
        @parse
        body = {$($body:tt)*},
        callback = $callback:ident ($($headers:tt)*)
    ) => {
        __parse_struct_body! {
            body = ($($body)*,),
            buff = ( attr [] ),
            fields = [],
            callback = $callback ($($headers)*)
        }
    };

    // Entry point for tuple structs
    (
        body = ($($body:tt)*),
        callback = $callback:ident ($($headers:tt)*)
    ) => {
        __parse_struct_body! {
            body = ($($body)*,),
            buff = ( attr [] ),
            fields = [],
            callback = $callback ($($headers)*)
        }
    };

    // parse attributes for fields (e.g. doc-comments)
    (
        body = (
            #[$($meta:tt)*]
            $($tail:tt)*),
        buff = ( attr [$($attr:tt)*] ),
        fields = $fields:tt,
        callback = $callback:ident ($($headers:tt)*)
    ) => {
        __parse_struct_body! {
            body = ($($tail)*),
            buff = ( attr [$($attr:tt)* #[$($meta)*]] ),
            fields = $fields,
            callback = $callback ($($headers)*)
        }
    };

    // silently skip visibility specifier
    (
        body = (
            pub $field_name:ident
            $($tail:tt)*),
        buff = $buff:tt,
        fields = $fields:tt,
        callback = $callback:ident ($($headers:tt)*)
    ) => {
        __parse_struct_body! {
            body = ($field_name $($tail)*),
            buff = $buff,
            fields = $fields,
            callback = $callback ($($headers)*)
        }
    };

    // Since we blindly add a comma to the end of the body, we might have a
    // double trailing comma.  If it's the only token left, that's what
    // happened. Strip it.
    (
        body = (,),
        buff = $buff:tt,
        fields = $fields:tt,
        callback = $callback:ident ($($headers:tt)*)
    ) => {
        __parse_struct_body! {
            body = (),
            buff = $buff,
            fields = $fields,
            callback = $callback ($($headers)*)
        }
    };

    // handle struct field and its type
    (
        body = ($field_name:ident : $field_ty:ty, $($tail:tt)*),
        buff = ( attr $field_attr:tt ),
        fields = [$($fields:tt)*],
        callback = $callback:ident ($($headers:tt)*)
    ) => {
        __parse_struct_body! {
            body = ($($tail)*),
            buff = ( attr [] ),
            fields = [$($fields)* {
                field_name: $field_name,
                field_ty: $field_ty,
                field_attr: $field_attr,
            }],
            callback = $callback ($($headers)*)
        }
    };

    // At this point we've parsed the entire body. We create the pattern
    // for destructuring, and pass all the information back to the main macro
    // to generate the final impl
    (
        body = (),
        buff = ( attr [] ),
        fields = $fields:tt,
        callback = $callback:ident ($($headers:tt)*)
    ) => {
        $callback! {
            $($headers)*
            fields = $fields,
        }
    };
}
