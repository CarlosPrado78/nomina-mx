use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use tauri::AppHandle;
use tauri::Manager;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Empresa {
    pub id: Option<i64>,
    pub rfc: String,
    pub razon_social: String,
    pub regimen_fiscal: String,
    pub cp: String,
    pub estado_isn: String,
    pub tasa_isn: f64,
    pub prima_riesgo_trabajo: f64,
    pub registro_patronal_imss: String,
}

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
    let row = sqlx::query_as::<_, (i64, String, String, String, String, String, f64, f64, String)>(
        "SELECT id, rfc, razon_social, regimen_fiscal, cp, estado_isn, tasa_isn, prima_riesgo_trabajo, registro_patronal_imss FROM empresas LIMIT 1",
    )
    .fetch_optional(&p).await.map_err(|e| e.to_string())?;
    Ok(row.map(|r| Empresa {
        id: Some(r.0), rfc: r.1, razon_social: r.2, regimen_fiscal: r.3, cp: r.4,
        estado_isn: r.5, tasa_isn: r.6, prima_riesgo_trabajo: r.7, registro_patronal_imss: r.8,
    }))
}