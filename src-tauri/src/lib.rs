pub mod commands;
pub mod calc;
pub mod db;
pub mod cfdi;
pub mod pdf;

use tauri::Manager;

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .setup(|app| {
            let app_dir = app.path().app_data_dir().expect("no app data dir");
            db::run_migrations(&app_dir)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
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
            commands::recalculo::recalcular_periodo,
            commands::recalculo::recalcular_trabajador_periodo,
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