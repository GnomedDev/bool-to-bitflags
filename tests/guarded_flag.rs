#[bool_to_bitflags::bool_to_bitflags]
#[derive(PartialEq, Debug)]
struct Test {
    #[cfg(False)]
    disabled_field: u8,
    enabled_field: u8,
    #[cfg(False)]
    disabled_flag: bool,
    enabled_flag: bool,
}

#[test]
fn test() {
    let mut can_construct = Test {
        enabled_field: 1,
        __generated_flags: TestGeneratedFlags::empty(),
    };

    can_construct.set_enabled_flag(true);

    let original: TestGeneratedOriginal = can_construct.into();
    assert_eq!(
        original,
        TestGeneratedOriginal {
            enabled_field: 1,
            enabled_flag: true
        }
    )
}
