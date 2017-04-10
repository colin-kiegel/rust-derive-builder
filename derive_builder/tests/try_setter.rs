#[macro_use]
extern crate derive_builder;

mod struct_level {
    use std::net::SocketAddr;
    use std::str::FromStr;
    
    #[derive(Debug, Builder)]
    #[builder(try_setter, setter(into))]
    struct Lorem {
        pub address: SocketAddr
    }
    
    #[test]
    fn infallible_set() {
        let _ = LoremBuilder::default().address(SocketAddr::from_str("1.2.3.4:80").unwrap()).build();
    }
}