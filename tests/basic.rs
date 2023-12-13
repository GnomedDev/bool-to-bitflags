#[bool_to_bitflags::bool_to_bitflags]
#[derive(Default, Clone, PartialEq, Eq, Debug, serde::Serialize, serde::Deserialize)]
struct Human {
    age: u8,
    is_epic: bool,
    /// Describes if the human is epic
    is_cool: bool,
}

#[test]
fn size() {
    assert_eq!(std::mem::size_of::<Human>(), 2);
}

#[test]
fn setters() {
    let mut example = Human::default();
    assert!(!example.is_cool());
    example.set_is_cool(true);
    assert!(example.is_cool());
}
