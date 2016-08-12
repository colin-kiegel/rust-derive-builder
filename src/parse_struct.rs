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
            fields = [],
            body = ($($body)*,),
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
            fields = [],
            body = ($($body)*,),
        }
    };

    // silently skip meta-tokens (e.g. doc-comments)
    (
        $headers:tt,
        callback = $callback:ident,
        fields = $fields:tt,
        body = (
            #$meta:tt
            $($tail:tt)*),
    ) => {
        __parse_struct_body! {
            $headers,
            callback = $callback,
            fields = $fields,
            body = ($($tail)*),
        }
    };

    // silently skip visibility specifier
    (
        $headers:tt,
        callback = $callback:ident,
        fields = $fields:tt,
        body = (
            pub
            $($tail:tt)*),
    ) => {
        __parse_struct_body! {
            $headers,
            callback = $callback,
            fields = $fields,
            body = ($($tail)*),
        }
    };

    // Since we blindly add a comma to the end of the body, we might have a
    // double trailing comma.  If it's the only token left, that's what
    // happened. Strip it.
    (
        $headers:tt,
        callback = $callback:ident,
        fields = $fields:tt,
        body = (,),
    ) => {
        __parse_struct_body! {
            $headers,
            callback = $callback,
            fields = $fields,
            body = (),
        }
    };

    // When we find #[column_name] followed by a type, handle the tuple struct
    // field
    (
        $headers:tt,
        callback = $callback:ident,
        fields = [$($fields:tt)*],
        body = (
            #[column_name($column_name:ident)]
            $field_ty:ty , $($tail:tt)*),
    ) => {
        __parse_struct_body! {
            $headers,
            callback = $callback,
            fields = [$($fields)* {
                column_name: $column_name,
                field_ty: $field_ty,
                field_kind: regular,
            }],
            body = ($($tail)*),
        }
    };

    // When we find #[column_name] followed by a named field, handle it
    (
        $headers:tt,
        callback = $callback:ident,
        fields = $fields:tt,
        body = (
            #[column_name($column_name:ident)]
            $field_name:ident : $($tail:tt)*),
    ) => {
        __parse_struct_body! {
            $headers,
            callback = $callback,
            fields = $fields,
            body = ($field_name as $column_name : $($tail)*),
        }
    };

    // If we got here and didn't have a #[column_name] attr,
    // then the column name is the same as the field name
    (
        $headers:tt,
        callback = $callback:ident,
        fields = $fields:tt,
        body = ($field_name:ident : $($tail:tt)*),
    ) => {
        __parse_struct_body! {
            $headers,
            callback = $callback,
            fields = $fields,
            body = ($field_name as $field_name : $($tail)*),
        }
    };

    // At this point we know the column and field name, handle the type
    (
        $headers:tt,
        callback = $callback:ident,
        fields = [$($fields:tt)*],
        body = ($field_name:ident as $column_name:ident : $field_ty:ty, $($tail:tt)*),
    ) => {
        __parse_struct_body! {
            $headers,
            callback = $callback,
            fields = [$($fields)* {
                field_name: $field_name,
                column_name: $column_name,
                field_ty: $field_ty,
                field_kind: regular,
            }],
            body = ($($tail)*),
        }
    };

    // When we reach a type with no column name annotation, handle the unnamed
    // tuple struct field. Since we require that either all fields are annotated
    // or none are, we could actually handle the whole body in one pass for this
    // case. However, anything using tuple structs without the column name
    // likely needs some ident per field to be useable and by handling each
    // field separately this way, the `field_kind` acts as a fresh ident each
    // time.
    (
        $headers:tt,
        callback = $callback:ident,
        fields = [$($fields:tt)*],
        body = ($field_ty:ty , $($tail:tt)*),
    ) => {
        __parse_struct_body! {
            $headers,
            callback = $callback,
            fields = [$($fields)* {
                field_ty: $field_ty,
                field_kind: bare,
            }],
            body = ($($tail)*),
        }
    };

    // At this point we've parsed the entire body. We create the pattern
    // for destructuring, and pass all the information back to the main macro
    // to generate the final impl
    (
        $headers:tt,
        callback = $callback:ident,
        fields = $fields:tt,
        body = (),
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
