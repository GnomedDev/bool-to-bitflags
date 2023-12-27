#[bool_to_bitflags::bool_to_bitflags(getter_prefix = "get_")]
#[derive(Default)]
struct TestNoGetter {
    bool_1: bool,
    bool_2: bool,
}

#[test]
fn no_setter() {
    let mut test = TestNoGetter::default();
    test.set_bool_1(true);
    assert!(test.get_bool_1());
}
