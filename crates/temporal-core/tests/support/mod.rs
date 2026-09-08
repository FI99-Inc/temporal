// Shared fixture construction is used by different integration-test stages.
#![allow(dead_code)]

pub mod bases;
mod builder;
pub mod negative;
pub mod variants;

pub use builder::Case;

pub fn all_cases() -> Vec<Case> {
    let bases = bases::all();
    let mut cases = bases.clone();
    cases.extend(variants::all(&bases));
    cases
}

pub fn case(name: &str) -> Case {
    all_cases()
        .into_iter()
        .find(|case| case.name == name)
        .unwrap_or_else(|| panic!("unknown documented case {name}"))
}
