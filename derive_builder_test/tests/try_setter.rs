#![cfg_attr(feature = "nightlytests", feature(try_from))]

use std::net::IpAddr;

#[macro_use]
extern crate derive_builder;

#[derive(Debug, Clone, Builder)]
#[builder(try_setter)]
pub struct Lorem {
    source: IpAddr,
    dest: IpAddr,
    name: String,
}

#[derive(Default, Debug, Clone, Builder)]
#[builder(default, setter(prefix = "set", into), try_setter)]
pub struct Ipsum {
    source: Option<IpAddr>,
    name: String,
}

fn main() { }