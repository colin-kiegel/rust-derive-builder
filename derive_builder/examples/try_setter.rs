//! This example illustrates the use of try-setters.
//! Tests are suppressed using a fake feature so that this doesn't break the build on stable.
#![cfg(feature = "try_from")]
#![feature(try_from)]

#[macro_use]
extern crate derive_builder;

use std::convert::TryFrom;
use std::net::{IpAddr, AddrParseError};
use std::str::FromStr;
use std::string::ToString;

/// Temporary newtype hack around lack of TryFrom implementations
/// in std. The rust-lang issue on the subject says that there will be a
/// blanket impl for everything that currently implements FromStr, which
/// will make this feature much more useful for input validation.
#[derive(Debug, Clone, PartialEq)]
pub struct MyAddr(IpAddr);

impl From<IpAddr> for MyAddr {
    fn from(v: IpAddr) -> Self {
        MyAddr(v)
    }
}

impl<'a> TryFrom<&'a str> for MyAddr {
    type Err = AddrParseError;

    fn try_from(v: &str) -> Result<Self, AddrParseError> {
        Ok(MyAddr(IpAddr::from_str(v)?))
    }
}

#[derive(Builder, Debug, PartialEq)]
#[builder(try_setter, setter(into))]
struct Lorem {
    pub name: String,
    pub addr: MyAddr,
}

fn main() {
    create("Jane", "1.2.3.4").unwrap();
    create("Bobby", "").unwrap_err();
}

fn create(name: &str, addr: &str) -> Result<Lorem, String> {
    // Fallible and infallible setters can be mixed freely when using
    // the mutable builder pattern.
    LoremBuilder::default()
        .name(name)
        .try_addr(addr).map_err(|e| e.to_string())?
        .build()
}