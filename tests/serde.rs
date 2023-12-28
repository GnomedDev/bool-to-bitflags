#[bool_to_bitflags::bool_to_bitflags]
#[derive(Debug, PartialEq, Clone, serde::Deserialize, serde::Serialize)]
struct Test {
    #[serde(default)]
    works: bool,
}

#[test]
fn serde() {
    let example_json = "{\"works\":true}";
    let example: Test = serde_json::from_str(example_json).unwrap();
    assert_eq!(example, TestGeneratedOriginal { works: true }.into());

    let serialized_example = serde_json::to_string(&example).unwrap();
    assert_eq!(serialized_example, example_json);
}

#[bool_to_bitflags::bool_to_bitflags]
#[derive(serde::Deserialize)]
#[serde(remote = "Self")]
struct TestRemote {
    to_invert: bool,
}

impl<'de> serde::Deserialize<'de> for TestRemoteGeneratedOriginal {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let mut test = Self::deserialize(deserializer)?;
        test.to_invert = !test.to_invert;
        Ok(test)
    }
}

#[test]
fn test_remote() {
    let test_json = "{\"to_invert\":false}";
    let test: TestRemote = serde_json::from_str(test_json).unwrap();

    assert!(test.to_invert());
}
