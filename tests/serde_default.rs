#[bool_to_bitflags::bool_to_bitflags]
#[derive(serde::Deserialize)]
struct Test {
    #[serde(default)]
    works: bool,
}
