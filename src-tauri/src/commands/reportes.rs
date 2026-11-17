use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use tauri::AppHandle;
use tauri::Manager;

#[derive(Debug, Serialize, Deserialize)]
pub struct ReporteImss {
    pub periodo_id: i64,
    pub total_obrero: f64,
    pub total_patronal: f64,
}

fn db_url(app: &AppHandle) -> String {
    let dir = app.path().app_data_dir().unwrap();
    format!("sqlite:{}", dir.join("nomina.db").display())
}

async fn pool(app: &AppHandle) -> Result<SqlitePool, String> {
    SqlitePool::connect(&db_url(app)).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn reporte_cuotas_imss(app: AppHandle, empresa_id: i64) -> Result<Vec<ReporteImss>, String> {
    let p = pool(&app).await?;
    let rows = sqlx::query_as::<_, (i64, f64, f64)>(
        "SELECT r.periodo_id, COALESCE(SUM(r.imss_obrero),0), COALESCE(SUM(r.cuota_patronal_imss),0) FROM recibos r JOIN periodos_nomina p ON p.id=r.periodo_id WHERE p.empresa_id=? GROUP BY r.periodo_id",
    )
    .bind(empresa_id).fetch_all(&p).await.map_err(|e| e.to_string())?;
    Ok(rows.into_iter().map(|r| ReporteImss { periodo_id: r.0, total_obrero: r.1, total_patronal: r.2 }).collect())
}