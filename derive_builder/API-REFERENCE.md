# API Reference

# Struct Options

Applied to the struct.

```rust
#[derive(Builder)]
#[builder(OPTIONS)]
pub struct Example {
    // ... fields elided
}
```

## build_fn

These attributes control the generated function that converts an instance of the builder into an instance of the deriving struct.

```rust
#[builder(build_fn(OPTIONS))]
struct Request {}
```

### error

Path of the error type returned by the build method.

```rust
enum RequestError {
    MalformedUrl(String),
    UninitializedField(&'static str),
}

// The generated build method will use this conversion to create
// an error when a required field has not been initialized.
impl From<derive_builder::UninitializedFieldError> for RequestError {
    fn from(e: derive_builder::UninitializedFieldError) -> Self {
        Self::UninitializedField(e.field_name())
    }
}

#[builder(build_fn(error = "RequestError"))]
struct Request {
    url: String
}
```

If `error` is not set, the following error enum will be automatically generated:

```rust
// deriving struct...
struct Request {}

// generated error
#[derive(Debug)]
#[non_exhaustive]
enum RequestBuilderError {
    UninitializedField(&'static str),
    ValidationError(String),
}
```

Note that the `ValidationError` variant is always included, even if `validate` was not specified; this ensures there will not be a visible API change from the later addition of a `validate` function.
If that variant is not desired, a custom error should be used instead.

It is not recommended to directly use `UninitializedFieldError` in your builder's API, as that tightly couples your builder's API to the `derive_builder` crate.

### name

Set the name of the generated build method.

If not specified, defaults to `build`.

```rust
#[builder(build_fn(name = "finish"))]
struct Request {}

fn demo() {
    RequestBuilder::default().finish();
}
```

### public/private

The visibility of the generated build method.

If not set, this will inherit the visibility of the builder struct.

Declaring both `public` and `private` will result in a compile error.

```rust
#[builder(build_fn(private))]
pub struct Request {}
```

### skip

Prevent generation of the build method.

**Note:** This is rarely desirable, as it requires manual implementation of all defaulting.
In most cases, you should instead either:

-   Use `#[builder(build_fn(validate = "..."))]` to add custom validation to the generated method OR
-   Use `#[builder(build_fn(private))]` to hide the generated method and then wrap it in your own public inherent build method which has any necessary additional logic

```rust
#[builder(build_fn(skip))]
struct Request {}
```

### validate

Path to a validation function with signature `(&Self) -> Result<(), TError>`, where `TError` converts to either the generated builder error or the type specified in `#[builder(build_fn(error = "..."))]`.

```rust
#[builder(build_fn(validate = "NoiseBuilder::not_loud_silence"))]
struct Noise {
    letter: char,
    volume: u8,
}

impl NoiseBuilder {
    fn not_loud_silence(self) -> Result<(), String> {
        match (self.letter.as_ref(), self.volume.as_ref()) {
            (Some(letter), Some(volume)) if letter.is_whitespace() && volume > 100 => Err("Loud silence".into()),
            (_, _) => Ok(())
        }
    }
}
```

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

The value to use for this field if the caller does not provide one.

If not set, the field will inherit from the struct-level `#[builder(default)]` declaration.
If that is also not set, then this field is required and will produce an `UninitializedFieldError` during the build step.

If set without a value, the `Default` impl for the field's data type will be used.

```rust
struct Request {
    // This will default to ""
    #[builder(default)]
    user: String
}
```

If set with a value, the value is a Rust expression that will be inserted when the default value is needed.
Note that unlike `serde`, this is not a path.

```rust
struct Request {
    // This is valid
    #[builder(default = "45"))]
    timeout_sec: u16,
    // Note that parentheses are required here
    #[builder(default = "infer_from_env()")]
    retry_limit: u8,
}
```

Unlike the struct-level default expression, field-level default expressions are only executed if the builder does not have a value for that field.

## setter

These attributes control the setter for an individual field.

Note: If `#[builder(setter(skip))]` has been declared on the struct, an individual field can override that to have its setter enabled by writing `#[builder(setter)]` with no additional properties.

### custom

This attribute causes the builder to include the field as an `Option`, but not to generate any setter.

```rust
struct Request {
    #[builder(setter(custom))]
    timeout_ms: u16
}

// Note that this is an impl block on RequestBuilder, not Request
impl RequestBuilder {
    fn timeout_sec(&mut self, timeout: u16) -> &mut Self {
        self.timeout_ms = Some(timeout * 1000);
        self
    }
}
```

### each

When a field is an extensible collection, using `each` generates a second setter for adding items to that collection.

This attribute can be a key-value pair, or a nested list.

```rust
struct Request {
    // Key-value pair (shorthand)
    #[builder(setter(each = "with_header"))]
    headers: Vec<Header>
}

struct Request {
    // Nested list (long-form)
    #[builder(setter(each(name = "with_header", into)))]
    headers: Vec<Header>
}
```

The long-form is necessary for specifying `into` on the each setter. If not specifying `into`, the two forms are equivalent.

`setter/each/into` has an analogous effect as `setter/into`; if set, the `each` setter will accept an iterable of items that can be converted into members of the extended collection's type.

### into

Make the setter accept a value that converts into that field's type, rather than requiring exactly the field's type.

```rust
struct Request {
    #[builder(setter(into))]
    method: String
}

fn demo() {
    // This works because `&'static str: Into<String>`
    RequestBuilder::default().method("get");
}
```

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

This attribute causes the builder to include the field as a `PhantomData`, and to not generate a setter.
If the field type does not impl `Default` and no field-level or struct-level default is provided, a compile error will be produced.

```rust
struct Request {
    #[builder(setter(skip))]
    http_client: Client,
}
```

### strip_option

If the field is of type `Option<T>`, make the generated setter accept `T` rather than `Option<T>`.

```rust
struct Request {
    #[builder(setter(strip_option))]
    keep_alive: Option<bool>,
    user: Option<String>,
    url: String,
}

fn demo() {
    RequestBuilder::default()
        .keep_alive(true)
        // strip_option was not applied to this field
        .user(Some("alice".into()))
        .url("https://crates.io".into());
}
```

Note that this setting does not work if a type alias is used for `Option`.

## public/private

...

## try_setter

...

## field

...
