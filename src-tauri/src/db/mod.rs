// Modulo de base de datos: helpers para obtener conexion y ejecutar migraciones.
// Las migraciones SQL viven en src-tauri/migrations y se registran en lib.rs
// mediante tauri_plugin_sql::Builder::add_migrations.

pub const DB_URL: &str = "sqlite:nomina.db";

pub fn db_path() -> std::path::PathBuf {
    // Resuelve a app data dir en runtime; aqui solo helper.
    std::path::PathBuf::from("nomina.db")
}