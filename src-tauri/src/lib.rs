pub mod commands;
pub mod calc;
pub mod db;
pub mod cfdi;
pub mod pdf;

use tauri::Manager;

pub fn run() {
    tauri::Builder::default()
        .plugin(
            tauri_plugin_sql::Builder::default()
                .add_migrations("sqlite:nomina.db", vec![
                    tauri_plugin_sql::Migration {
                        version: 1,
                        description: "init",
                        sql: include_str!("../migrations/0001_init.sql"),
                        kind: tauri_plugin_sql::MigrationKind::Up,
                    },
                    tauri_plugin_sql::Migration {
                        version: 2,
                        description: "seed fiscal 2026",
                        sql: include_str!("../migrations/0002_seed_fiscal_2026.sql"),
                        kind: tauri_plugin_sql::MigrationKind::Up,
                    },
                ])
                .build(),
        )
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .setup(|app| {
            let app_dir = app.path().app_data_dir().expect("no app data dir");
            std::fs::create_dir_all(&app_dir).ok();
            // Run migrations on startup
            let db_path = app_dir.join("nomina.db");
            let _ = db_path;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::empresa::guardar_empresa,
            commands::empresa::obtener_empresa,
            commands::trabajador::guardar_trabajador,
            commands::trabajador::listar_trabajadores,
            commands::trabajador::obtener_trabajador,
            commands::nomina::calcular_recibo,
            commands::nomina::guardar_periodo,
            commands::nomina::listar_periodos,
            commands::finiquito::calcular_finiquito,
            commands::finiquito::calcular_liquidacion,
            commands::fiscal::obtener_parametros,
            commands::fiscal::guardar_parametro,
            commands::fiscal::listar_estados_isn,
            commands::cfdi_cmd::generar_cfdi,
            commands::reportes::reporte_cuotas_imss,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}