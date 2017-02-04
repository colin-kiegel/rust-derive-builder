#[macro_use] extern crate derive_builder;

#[derive(Debug, PartialEq, Default, Builder, Clone)]
#[builder(pattern="immutable")]
struct Lorem {
    immutable: u32,
    #[builder(pattern="mutable")]
    mutable_override: u32,
}

#[derive(Debug, PartialEq, Default, Builder, Clone)]
#[builder(pattern="mutable")]
struct Ipsum {
    mutable: u32,
    #[builder(pattern="owned")]
    owned_override: u32,
}

#[derive(Debug, PartialEq, Default, Builder, Clone)]
#[builder(pattern="owned")]
struct Dolor {
    #[builder(pattern="immutable")]
    immutable_override: u32,
    owned: u32,
}

#[derive(Debug, PartialEq, Default, Builder, Clone)]
struct Sit {
    default: u32,
}

type ImmutableSetter<T, U> = fn(&T, U) -> T;
type OwnedSetter<T, U> = fn(T, U) -> T;
type MutableSetter<T, U> = fn(&mut T, U) -> &mut T;

#[test]
fn mutable_by_default() {
    // the setter must have the correct signature
    let mutable_setter: MutableSetter<Sit, u32> = Sit::default;

    let mut old = <Sit as Default>::default();
    mutable_setter(&mut old, 42);
    assert_eq!(old.default, 42);
}

#[test]
fn mutable() {
    // the setter must have the correct signature
    let mutable_setter: MutableSetter<Ipsum, u32> = Ipsum::mutable;

    let mut old = Ipsum::default();
    mutable_setter(&mut old, 42);
    assert_eq!(old.mutable, 42);
}

#[test]
fn mutable_override() {
    // the setter must have the correct signature
    let mutable_setter: MutableSetter<Lorem, u32> = Lorem::mutable_override;

    let mut old = Lorem::default();
    mutable_setter(&mut old, 42);
    assert_eq!(old.mutable_override, 42);
}

#[test]
fn immutable() {
    // the setter must have the correct signature
    let immutable_setter: ImmutableSetter<Lorem, u32> = Lorem::immutable;

    let old = Lorem::default();
    let new = immutable_setter(&old, 42);
    assert_eq!(new.immutable, 42);
}

#[test]
fn immutable_override() {
    // the setter must have the correct signature
    let immutable_setter: ImmutableSetter<Dolor, u32> = Dolor::immutable_override;

    let old = Dolor::default();
    let new = immutable_setter(&old, 42);
    assert_eq!(new.immutable_override, 42);
}

#[test]
fn owned() {
    // the setter must have the correct signature
    let owned_setter: OwnedSetter<Dolor, u32> = Dolor::owned;

    let old = Dolor::default();
    let new = owned_setter(old, 42);
    assert_eq!(new.owned, 42);
}

#[test]
fn owned_override() {
    // the setter must have the correct signature
    let owned_setter: OwnedSetter<Ipsum, u32> = Ipsum::owned_override;

    let old = Ipsum::default();
    let new = owned_setter(old, 42);
    assert_eq!(new.owned_override, 42);
}
