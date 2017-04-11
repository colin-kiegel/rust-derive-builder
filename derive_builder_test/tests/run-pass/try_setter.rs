#![cfg_attr(feature = "nightlytests", feature(try_from))]

#[macro_use]
extern crate derive_builder;

#[allow(unused_imports)]
mod struct_level {
    #[cfg(feature = "try_from")]
    use std::convert::TryFrom;
    
    use std::net::{IpAddr, AddrParseError};
    use std::str::FromStr;
    use std::string::ToString;

    #[derive(Debug, Clone, PartialEq)]
    pub struct MyAddr(IpAddr);

    impl From<IpAddr> for MyAddr {
        fn from(v: IpAddr) -> Self {
            MyAddr(v)
        }
    }

    #[cfg(feature = "nightlytests")]
    impl<'a> TryFrom<&'a str> for MyAddr {
        type Err = AddrParseError;

        fn try_from(v: &str) -> Result<Self, Self::Err> {
            Ok(MyAddr(v.parse()?))
        }
    }

    #[derive(Debug, PartialEq, Builder)]
    #[builder(try_setter, setter(into))]
    struct Lorem {
        pub source: MyAddr,
        pub dest: MyAddr,
    }

    #[test]
    fn infallible_set() {
        let _ = LoremBuilder::default()
            .source(IpAddr::from_str("1.2.3.4").unwrap())
            .dest(IpAddr::from_str("0.0.0.0").unwrap())
            .build();
    }

    #[test]
    #[cfg(feature = "nightlytests")]
    fn fallible_set() {
        let mut builder = LoremBuilder::default();
        let try_result = builder.try_source("1.2.3.4");
        let built = try_result.expect("Passed well-formed address")
            .dest(IpAddr::from_str("0.0.0.0").unwrap())
            .build()
            .unwrap();
        assert_eq!(built, exact_helper().unwrap());
    }

    // Allow dead code here since the test that uses this depends on the try_setter feature.
    #[cfg_attr(not(feature = "nightlytests"), allow(dead_code))]
    fn exact_helper() -> Result<Lorem, String> {
        LoremBuilder::default()
            .source(IpAddr::from_str("1.2.3.4").unwrap())
            .dest(IpAddr::from_str("0.0.0.0").unwrap())
            .build()
    }

    #[cfg(feature = "nightlytests")]
    fn try_helper() -> Result<Lorem, String> {
        LoremBuilder::default()
            .try_source("1.2.3.4").map_err(|e| e.to_string())?
            .try_dest("0.0.0.0").map_err(|e| e.to_string())?
            .build()
    }

    #[test]
    #[cfg(feature = "nightlytests")]
    fn with_helper() {
        assert_eq!(exact_helper().unwrap(), try_helper().unwrap());
    }
}