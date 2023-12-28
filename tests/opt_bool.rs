#[bool_to_bitflags::bool_to_bitflags]
#[derive(Clone, Default, serde::Deserialize, serde::Serialize)]
struct OptionalBool {
    opt_bool: Option<bool>,
    normal: bool,
}

#[test]
fn test() {
    let mut example = OptionalBool::default();
    assert!(example.opt_bool().is_none());

    example.set_opt_bool(Some(false));
    assert_eq!(example.opt_bool(), Some(false));

    example.set_opt_bool(Some(true));
    assert_eq!(example.opt_bool(), Some(true));

    assert_eq!(std::mem::size_of::<OptionalBool>(), 1)
}
