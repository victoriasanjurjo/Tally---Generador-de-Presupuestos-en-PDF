// Pruebas de integración y verificación de reglas de negocio para Tally.
// Copia espejo de verificación para ejecutar con `cargo test`.

#[test]
fn project_structure_compliance() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    assert!(root.join("docs").is_dir(), "Debe existir la carpeta docs/");
    assert!(root.join("tests").is_dir(), "Debe existir la carpeta tests/");
    assert!(root.join("config").is_dir(), "Debe existir la carpeta config/");
    assert!(root.join("assets").is_dir(), "Debe existir la carpeta assets/");
    assert!(root.join("outputs").is_dir(), "Debe existir la carpeta outputs/");
    assert!(root.join("src").is_dir(), "Debe existir la carpeta src/");
    assert!(root.join("LICENSE.txt").is_file(), "Debe existir LICENSE.txt");
    assert!(root.join(".gitignore").is_file(), "Debe existir .gitignore");
}

