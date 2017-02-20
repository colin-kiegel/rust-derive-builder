// https://github.com/colin-kiegel/rust-derive-builder/issues/15
#[macro_use]
extern crate derive_builder;

#[derive(Debug, PartialEq, Default, Builder, Clone)]
#[builder(setter(skip="false"))]
struct SetterOptOut {
    setter_present_by_explicit_default: u32,
    #[builder(setter(skip="true"))]
    setter_skipped_by_explicit_opt_out: u32,
    #[builder(setter(skip))]
    setter_skipped_by_shorthand_opt_out: u32,
}

#[derive(Debug, PartialEq, Default, Builder, Clone)]
#[builder(setter(skip))]
struct SetterOptIn {
    setter_skipped_by_shorthand_default: u32,
    #[builder(setter(skip="false"))]
    setter_present_by_explicit_opt_in: u32,
    #[builder(setter)]
    setter_present_by_shorthand_opt_in: u32,
}

// compile test
#[allow(dead_code)]
impl SetterOptOut {
    // only possible if setter was skipped
    fn setter_skipped_by_explicit_opt_out() {}
    // only possible if setter was skipped
    fn setter_skipped_by_shorthand_opt_out() {}
}

// compile test
#[allow(dead_code)]
impl SetterOptIn {
    // only possible if setter was skipped
    fn setter_skipped_by_shorthand_default() {}
}

#[test]
fn setter_opt_out() {
    let x: SetterOptOut = SetterOptOutBuilder::default()
        .setter_present_by_explicit_default(42u32)
        .build()
        .unwrap();

    assert_eq!(x,
               SetterOptOut {
                   setter_present_by_explicit_default: 42,
                   setter_skipped_by_explicit_opt_out: 0,
                   setter_skipped_by_shorthand_opt_out: 0,
               });
}

#[test]
fn setter_opt_in() {
    let x: SetterOptIn = SetterOptInBuilder::default()
        .setter_present_by_explicit_opt_in(47u32)
        .setter_present_by_shorthand_opt_in(11u32)
        .build()
        .unwrap();

    assert_eq!(x,
               SetterOptIn {
                   setter_skipped_by_shorthand_default: 0,
                   setter_present_by_explicit_opt_in: 47,
                   setter_present_by_shorthand_opt_in: 11,
               });
}
