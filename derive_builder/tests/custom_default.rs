#[macro_use]
extern crate pretty_assertions;
#[macro_use]
extern crate derive_builder;

mod field_level {
    #[derive(Debug, PartialEq, Default, Builder, Clone)]
    struct Lorem {
        required: String,
        #[builder(default)]
        explicit_default: String,
        #[builder(default="\"foo\".to_string()")]
        escaped_default: String,
        #[builder(default=r#"format!("Hello {}!", "World")"#)]
        raw_default: String,
        #[builder(default=r#"format!("{}-{}-{}-{}",
                             Clone::clone(self.required
                                .as_ref()
                                .ok_or("required must be initialized")?),
                             match self.explicit_default { Some(ref x) => x, None => "EMPTY" },
                             self.escaped_default.as_ref().map(|x| x.as_ref()).unwrap_or("EMPTY"),
                             if let Some(ref x) = self.raw_default { x } else { "EMPTY" })"#)]
        computed_default: String,
    }

    #[test]
    #[should_panic(expected="`required` must be initialized")]
    fn panic_if_uninitialized() {
        LoremBuilder::default().build().unwrap();
    }

    #[test]
    fn custom_default() {
        let x = LoremBuilder::default()
            .required("ipsum".to_string())
            .build()
            .unwrap();

        assert_eq!(x, Lorem {
            required: "ipsum".to_string(),
            explicit_default: "".to_string(),
            escaped_default: "foo".to_string(),
            raw_default: "Hello World!".to_string(),
            computed_default: "ipsum-EMPTY-EMPTY-EMPTY".to_string(),
        });
    }

    #[test]
    fn builder() {
        let x = LoremBuilder::default()
            .required("ipsum".to_string())
            .explicit_default("lorem".to_string())
            .escaped_default("dolor".to_string())
            .raw_default("sit".to_string())
            .build()
            .unwrap();

        assert_eq!(x, Lorem {
            required: "ipsum".to_string(),
            explicit_default: "lorem".to_string(),
            escaped_default: "dolor".to_string(),
            raw_default: "sit".to_string(),
            computed_default: "ipsum-lorem-dolor-sit".to_string(),
        });
    }
}

mod struct_level {
    #[derive(Debug, PartialEq, Default, Builder, Clone)]
    #[builder(default)]
    struct Lorem {
        implicit_default: String,
        #[builder(default)]
        explicit_default: String,
        #[builder(default="\"foo\".to_string()")]
        escaped_default: String,
        #[builder(default=r#"format!("Hello {}!", "World")"#)]
        raw_default: String,
    }

    #[test]
    fn implicit_default() {
        let x = LoremBuilder::default()
            .build()
            .unwrap();

        assert_eq!(x, Lorem {
            implicit_default: "".to_string(),
            explicit_default: "".to_string(),
            escaped_default: "foo".to_string(),
            raw_default: "Hello World!".to_string(),
        });
    }

    #[test]
    fn builder() {
        let x = LoremBuilder::default()
            .implicit_default("ipsum".to_string())
            .explicit_default("lorem".to_string())
            .escaped_default("dolor".to_string())
            .raw_default("sit".to_string())
            .build()
            .unwrap();

        assert_eq!(x, Lorem {
            implicit_default: "ipsum".to_string(),
            explicit_default: "lorem".to_string(),
            escaped_default: "dolor".to_string(),
            raw_default: "sit".to_string(),
        });
    }
}

#[cfg(feature = "struct_default")]
mod struct_impl {
    #[derive(Debug, Clone, PartialEq, Eq, Builder)]
    #[builder(default)]
    struct Ipsum {
        not_type_default: Option<u16>,
        also_custom: bool,
        is_type_default: String,
    }
    
    impl Default for Ipsum {
        fn default() -> Self {
            Ipsum {
                not_type_default: Some(20),
                also_custom: true,
                is_type_default: Default::default(),
            }
        }
    }
    
    #[test]
    fn defaults_are_equal() {
        assert_eq!(Ok(Ipsum::default()), IpsumBuilder::default().build());
    }
    
    #[test]
    fn overrides_work() {
        let ipsum = IpsumBuilder::default()
            .not_type_default(None)
            .build()
            .expect("Struct-level default makes all fields optional");
        assert_eq!(None, ipsum.not_type_default);
    }
}

#[cfg(feature = "struct_default")]
mod struct_explicit {
    fn helper() -> Dolor {
        Dolor {
            not_type_default: Some(20)
        }
    }
    
    #[derive(Debug, PartialEq, Builder)]
    #[builder(default = "helper")]
    struct Dolor {
        not_type_default: Option<u16>
    }
    
    #[test]
    fn defaults_are_equal() {
        assert_eq!(Ok(helper()), DolorBuilder::default().build());
    }
}

mod local_foolishness {
#[derive(Debug, PartialEq, Builder)]
struct Dolor {
    
    #[builder(default)]
    msg: String,
    
    /// This will print "Hi!" if z_failure is not explicitly initialized.
    #[builder(default = "{println!(\"Hi!\"); Some(\"woot\".to_string())}")]
    z_failure: Option<String>
}
    
    #[test]
    #[cfg(feature = "struct_default")]
    fn defaults() {
        assert_eq!(Some("woot".to_string()), DolorBuilder::default().build().unwrap().z_failure);
    }
}