#[macro_use]
extern crate derive_builder;

mod struct_default {
    #[derive(Debug, Default, PartialEq, Builder)]
    #[builder(default)]
    struct Hello {
        world: String
    }
    
    #[test]
    fn defaults() {
        assert_eq!(Ok(Hello::default()), HelloBuilder::default().build());
    }
}

mod field_default {
    
    #[derive(Debug, Default, PartialEq, Builder)]
    struct Goodbye {
        #[builder(default)]
        world: String
    }
    
    #[test]
    fn pass() {}
}