#[bool_to_bitflags::bool_to_bitflags(owning_setters)]
#[derive(Default)]
struct Test {
    bool_1: bool,
    bool_2: bool,
}

#[test]
fn test() {
    assert!(Test::default().set_bool_1(true).bool_1())
}
