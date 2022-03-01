# API Reference

# Struct Options

Applied to the struct.

```rust
#[derive(Builder)]
#[builder(OPTIONS HERE)]
pub struct Example {
    // ... fields elided
}
```

## build_fn

These attributes control the generated function that converts an instance of the builder into an instance of the deriving struct.

### error

...

### name

...

### public/private

...

### skip

...

### validate

...

## default

Specify the instance of the deriving struct that will be used to get default values for unspecified fields.

If not set, no struct-level default will be used.

Specifying this attribute without a value will result in the struct's `Default` impl being used.

```rust
#[builder(default)]
struct Request {}
```

If a value is specified, then that will be interpreted as a **block** of Rust code. This is not like `serde`, which interprets a provided value as a path.

In the example below, note the inclusion of parentheses in the default expression.

```rust
fn path_to_default() -> Request {
    Request {
        a_field: "indeed"
    }
}

#[builder(default = "path_to_default()")]
struct Request {
    a_field: &'static str,
}
```

## derive

List of derives to emit on the generated builder.

Note that the `pattern` attribute may impose a `Clone` constraint on the builder; that does not need to be declared separately here.

```rust
#[builder(derive(Serialize))]
struct Request {}
```

## field

...

## name

Changes the name of the emitted builder.

If not set, defaults to the name of the struct followed by `Builder`.

```rust
#[builder(name = "RequestOptions")]
struct Request {}
```

## pattern

...

## public/private

Change the visibility of the builder struct.

If neither `public` nor `private` are declared, the builder struct inherits the visibility of the deriving struct.

Declaring both `public` and `private` will result in a compile-time error.

```rust
#[builder(public)]
struct Request {}
```

## setter

These attributes set the defaults for setter generation.
They can be overridden by setting the same attribute on an individual field.

### into

Make each setter accept a value that converts into that field's type, rather than requiring exactly the field's type.

```rust
#[builder(setter(into))]
struct Request {
    method: String
}

fn demo() {
    // This works because there is a conversion from &str to String
    RequestBuilder::default().method("get");
}
```

### prefix

A prefix added to each field's name to generate the setter method name.

A value of "" is the same as omitting the property.

```rust
#[builder(prefix = "set_")]
pub struct Request {
    method: &'static str
}

fn demo() {
    RequestBuilder::default().set_method("get");
}
```

### skip

...

### strip_option

Make setters for fields of type `Option<T>` will accept `T` rather than `Option<T>`.

```rust
#[builder(setter(strip_option))]
struct Request {
    keep_alive: Option<bool>,
    user: Option<String>,
    url: String,
}

fn demo() {
    RequestBuilder::default()
        .keep_alive(true)
        .user("alice".into())
        .url("https://crates.io".into());
}
```

## try_setter

...

# Field Options

## default

...

## setter

These attributes control the setter for an individual field.

Note: If `#[builder(setter(skip))]` has been declared on the struct, an individual field can override that to have its setter enabled by writing `#[builder(setter)]` with no additional properties.

### custom

...

### each

...

### into

...

### prefix

A prefix added to the field's name to generate the setter method name.

A value of "" is the same as omitting the property.

```rust
pub struct Request {
    timeout: u64,
    #[builder(prefix = "set_")]
    method: &'static str
}

fn result() {
    RequestBuilder::default()
        .timeout(10)
        .set_method("get");
}
```

### skip

...

### strip_option

...

## public/private

...

## try_setter

...

## field

...
