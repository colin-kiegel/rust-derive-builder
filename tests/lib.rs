#[macro_use] extern crate custom_derive;
#[macro_use] extern crate derive_builder;

use std::convert::From;

#[derive(PartialEq, Default, Debug, Clone)]
struct Uuid(i32);
#[derive(PartialEq, Default, Debug, Clone)]
struct Authentication(i32);

impl From<i32> for Uuid {
    fn from(x: i32) -> Uuid {
        Uuid(x)
    }
}

impl From<i32> for Authentication {
    fn from(x: i32) -> Authentication {
        Authentication(x)
    }
}

custom_derive!{
    #[derive(Default)]
    struct Channel {
        id: Uuid,
        token: Authentication,
        special_info: i32
    }
}

// TODO: make this auto-generated code :-)
// --- SNIP ---
impl Channel {
    fn new<T1: Into<Uuid>, T2: Into<Authentication>> (id: T1, token: T2) -> Channel {
        Channel {
            id: id.into(),
            token: token.into(),
            special_info: 42i32,
        }
    }

    fn id<T: Into<Uuid>> (mut self, id: T) -> Self {
        self.id = id.into();

        self
    }

    fn token<T: Into<Authentication>> (mut self, token: T) -> Self {
        self.token = token.into();

        self
    }

    fn special_info<T: Into<i32>> (mut self, special_info: T) -> Self {
        self.special_info = special_info.into();

        self
    }
}
// --- SNAP ---

#[test]
fn contructor() {
    let id = Uuid(0);
    let raw_token = 1;

    let ch = Channel::new(id.clone(), raw_token);
    assert_eq!(ch.id, id);
    assert_eq!(ch.token, raw_token.into());
}

#[test]
fn custom_default() {
    let ch = Channel::new(0, 0);

    assert_eq!(ch.special_info, 42i32);
}

#[test]
fn default_trait() {
    let ch = Channel::default();

    assert_eq!(ch.special_info, i32::default());
}

#[test]
fn setters() {
    let id = Uuid(0);
    let raw_token = 1;
    let raw_special_info = 255u8;

    let ch = Channel::default()
        .id(id.clone())
        .token(raw_token)
        .special_info(raw_special_info);

    assert_eq!(ch.id, id);
    assert_eq!(ch.token, raw_token.into());
    assert_eq!(ch.special_info, raw_special_info.into());
}
