#[bool_to_bitflags::bool_to_bitflags]
#[derive(Default, Clone, PartialEq, Eq, Debug, serde::Serialize, serde::Deserialize)]
struct Human {
    age: u8,
    is_epic: bool,
    /// Describes if the human is epic
    is_cool: bool,
}

impl Human {
    pub fn new(age: u8, is_epic: bool, is_cool: bool) -> Self {
        let mut this = Self::default();
        this.set_is_epic(is_epic);
        this.set_is_cool(is_cool);
        this.age = age;
        this
    }
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

#[test]
fn serde() {
    let example_json = "{\"age\":18,\"is_epic\":true,\"is_cool\":false}";
    let example: Human = serde_json::from_str(example_json).unwrap();
    assert_eq!(example, Human::new(18, true, false));

    let serialized_example = serde_json::to_string(&example).unwrap();
    assert_eq!(serialized_example, example_json);
}
