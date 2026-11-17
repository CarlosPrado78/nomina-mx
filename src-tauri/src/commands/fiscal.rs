use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use tauri::AppHandle;
use tauri::Manager;

#[derive(Debug, Serialize, Deserialize)]
pub struct ParametroFiscal {
    pub id: Option<i64>,
    pub anio: i64,
    pub tipo: String,
    pub vigencia_inicio: String,
    pub vigencia_fin: String,
    pub datos_json: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EstadoIsn {
    pub estado: String,
    pub tasa: f64,
}

fn db_url(app: &AppHandle) -> String {
    let dir = app.path().app_data_dir().unwrap();
    format!("sqlite:{}", dir.join("nomina.db").display())
}

async fn pool(app: &AppHandle) -> Result<SqlitePool, String> {
    SqlitePool::connect(&db_url(app)).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn obtener_parametros(app: AppHandle, anio: i64, tipo: String) -> Result<Option<ParametroFiscal>, String> {
    let p = pool(&app).await?;
    let row = sqlx::query_as::<_, (i64, i64, String, String, String, String)>(
        "SELECT id, anio, tipo, vigencia_inicio, vigencia_fin, datos_json FROM tablas_fiscales WHERE anio=? AND tipo=? LIMIT 1",
    )
    .bind(anio).bind(&tipo).fetch_optional(&p).await.map_err(|e| e.to_string())?;
    Ok(row.map(|r| ParametroFiscal { id: Some(r.0), anio: r.1, tipo: r.2, vigencia_inicio: r.3, vigencia_fin: r.4, datos_json: r.5 }))
}

#[tauri::command]
pub async fn guardar_parametro(app: AppHandle, param: ParametroFiscal) -> Result<i64, String> {
    let p = pool(&app).await?;
    if let Some(id) = param.id {
        sqlx::query("UPDATE tablas_fiscales SET anio=?, tipo=?, vigencia_inicio=?, vigencia_fin=?, datos_json=? WHERE id=?")
            .bind(param.anio).bind(&param.tipo).bind(&param.vigencia_inicio).bind(&param.vigencia_fin).bind(&param.datos_json).bind(id)
            .execute(&p).await.map_err(|e| e.to_string())?;
        Ok(id)
    } else {
        let res = sqlx::query("INSERT INTO tablas_fiscales (anio, tipo, vigencia_inicio, vigencia_fin, datos_json) VALUES (?,?,?,?,?)")
            .bind(param.anio).bind(&param.tipo).bind(&param.vigencia_inicio).bind(&param.vigencia_fin).bind(&param.datos_json)
            .execute(&p).await.map_err(|e| e.to_string())?;
        Ok(res.last_insert_rowid())
    }
}

#[tauri::command]
pub async fn listar_estados_isn(app: AppHandle) -> Result<Vec<EstadoIsn>, String> {
    let p = pool(&app).await?;
    let row = sqlx::query_as::<_, (String,)>(
        "SELECT datos_json FROM tablas_fiscales WHERE tipo='tasa_isn_estados' ORDER BY anio DESC LIMIT 1",
    )
    .fetch_optional(&p).await.map_err(|e| e.to_string())?;
    if let Some((json,)) = row {
        let v: serde_json::Value = serde_json::from_str(&json).unwrap_or_default();
        let mut out = vec![];
        if let Some(obj) = v.as_object() {
            for (k, val) in obj {
                out.push(EstadoIsn { estado: k.clone(), tasa: val.as_f64().unwrap_or(0.0) });
            }
        }
        out.sort_by(|a, b| a.estado.cmp(&b.estado));
        Ok(out)
    } else {
        Ok(vec![])
    }
}