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
                    tauri_plugin_sql::Migration {
                        version: 3,
                        description: "bloque B: parametros de horas extra",
                        sql: include_str!("../migrations/0003_horas_extra.sql"),
                        kind: tauri_plugin_sql::MigrationKind::Up,
                    },
                    tauri_plugin_sql::Migration {
                        // El archivo se llama 0003_audit_log.sql por la
                        // definición del índice, pero la versión 3 ya está
                        // ocupada por horas extra.
                        version: 4,
                        description: "C3: log de auditoria inmutable",
                        sql: include_str!("../migrations/0003_audit_log.sql"),
                        kind: tauri_plugin_sql::MigrationKind::Up,
                    },
                    tauri_plugin_sql::Migration {
                        version: 5,
                        description: "C4: multi-empresa con seleccion persistida",
                        sql: include_str!("../migrations/0004_multi_empresa.sql"),
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
            commands::empresas::listar_empresas,
            commands::empresas::cambiar_empresa_activa,
            commands::empresas::crear_empresa,
            commands::trabajador::guardar_trabajador,
            commands::trabajador::listar_trabajadores,
            commands::trabajador::obtener_trabajador,
            commands::importar::importar_trabajadores_csv,
            commands::nomina::calcular_recibo,
            commands::nomina::guardar_periodo,
            commands::nomina::listar_periodos,
            commands::nomina::guardar_recibo,
            commands::finiquito::calcular_finiquito,
            commands::finiquito::calcular_liquidacion,
            commands::fiscal::obtener_parametros,
            commands::fiscal::guardar_parametro,
            commands::fiscal::listar_estados_isn,
            commands::cfdi_cmd::generar_cfdi,
            commands::reportes::reporte_cuotas_imss,
            commands::backup::backup_db,
            commands::backup::restore_db,
            commands::backup::listar_backups,
            commands::auditoria::listar_auditoria,
            commands::auditoria::configurar_usuario_auditoria,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}