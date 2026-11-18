// C4 — La ficha de empresa mostrada/editada es siempre la activa.
// El tipo `Empresa` y la resolución de la activa viven en `empresas.rs`.

use crate::commands::empresas::{obtener_empresa_activa_pool, Empresa};
use sqlx::SqlitePool;
use tauri::AppHandle;
use tauri::Manager;

fn db_url(app: &AppHandle) -> String {
    let dir = app.path().app_data_dir().unwrap();
    format!("sqlite:{}", dir.join("nomina.db").display())
}

async fn pool(app: &AppHandle) -> Result<SqlitePool, String> {
    SqlitePool::connect(&db_url(app)).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn guardar_empresa(app: AppHandle, empresa: Empresa) -> Result<i64, String> {
    let p = pool(&app).await?;
    if let Some(id) = empresa.id {
        sqlx::query(
            "UPDATE empresas SET rfc=?, razon_social=?, regimen_fiscal=?, cp=?, estado_isn=?, tasa_isn=?, prima_riesgo_trabajo=?, registro_patronal_imss=? WHERE id=?",
        )
        .bind(&empresa.rfc).bind(&empresa.razon_social).bind(&empresa.regimen_fiscal)
        .bind(&empresa.cp).bind(&empresa.estado_isn).bind(empresa.tasa_isn)
        .bind(empresa.prima_riesgo_trabajo).bind(&empresa.registro_patronal_imss).bind(id)
        .execute(&p).await.map_err(|e| e.to_string())?;
        Ok(id)
    } else {
        let res = sqlx::query(
            "INSERT INTO empresas (rfc, razon_social, regimen_fiscal, cp, estado_isn, tasa_isn, prima_riesgo_trabajo, registro_patronal_imss) VALUES (?,?,?,?,?,?,?,?)",
        )
        .bind(&empresa.rfc).bind(&empresa.razon_social).bind(&empresa.regimen_fiscal)
        .bind(&empresa.cp).bind(&empresa.estado_isn).bind(empresa.tasa_isn)
        .bind(empresa.prima_riesgo_trabajo).bind(&empresa.registro_patronal_imss)
        .execute(&p).await.map_err(|e| e.to_string())?;
        Ok(res.last_insert_rowid())
    }
}

#[tauri::command]
pub async fn obtener_empresa(app: AppHandle) -> Result<Option<Empresa>, String> {
    let p = pool(&app).await?;
    obtener_empresa_activa_pool(&p).await
}
