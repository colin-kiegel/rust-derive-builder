Builder for `{struct_name}`.

# Examples

```rust
let builder = <{builder_name} as Default>::default();
// .. call some setters on `builder`
let _x: Result<{struct_name}, String> = builder.build();
```
