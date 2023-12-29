#[bool_to_bitflags::bool_to_bitflags(owning_setters)]
#[derive(Default)]
struct ManyBools {
    bool_1: bool,
    bool_2: bool,
    bool_3: bool,
    bool_4: bool,
    bool_5: bool,
    bool_6: bool,
    bool_7: bool,
    bool_8: bool,
    bool_9: bool,
    bool_10: bool,
    bool_11: bool,
    bool_12: bool,
    bool_13: bool,
    bool_14: bool,
    bool_15: bool,
    bool_16: bool,
    bool_17: bool,
}

#[test]
fn test() {
    let test = ManyBools::default()
        .set_bool_1(true)
        .set_bool_9(true)
        .set_bool_17(true);

    assert!(test.bool_1());
    assert!(test.bool_9());
    assert!(test.bool_17());
    assert!(!test.bool_16());
}
