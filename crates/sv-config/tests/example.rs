//! `config/example.toml` no puede quedar desactualizado.
//!
//! Sus valores activos deben ser los valores por defecto. Si se cambia un valor
//! por defecto o se añade un campo, esta prueba obliga a actualizar el ejemplo.

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
            "el ejemplo no menciona el ajuste `{}`",
            key.trim()
        );
    }
}
