#[derive(Clone, serde::Serialize)]
struct NonCopy {}

#[bool_to_bitflags::bool_to_bitflags]
#[derive(Clone, serde::Serialize)]
struct Test {
    non_copy: NonCopy,
    is_epic: bool,
}
