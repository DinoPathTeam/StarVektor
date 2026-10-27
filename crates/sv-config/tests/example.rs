//! `config/example.toml` cannot be allowed to go out of date.
//!
//! Its active values must be the defaults. If a default value is changed
//! or a field is added, this test forces the example to be updated.

use sv_config::Config;

const EXAMPLE: &str = include_str!("../../../config/example.toml");

#[test]
fn the_example_file_equals_the_defaults() {
    assert_eq!(Config::from_toml_str(EXAMPLE).unwrap(), Config::default());
}

#[test]
fn the_example_documents_every_setting() {
    let default = Config::default().to_toml_string().unwrap();
    for line in default.lines() {
        let Some((key, _)) = line.split_once(" = ") else {
            continue;
        };
        assert!(
            EXAMPLE.contains(key.trim()),
            "the example does not mention the setting `{}`",
            key.trim()
        );
    }
}
