// C3 — Log de auditoría inmutable.
//
// Fuentes: CFF art. 28 (conservación de información y registros) y LSS art. 15
// (registro de movimientos afiliatorios). `timestamp` usa CURRENT_TIMESTAMP de
// SQLite, que es UTC. Los triggers guardan el snapshot en json_object() y este
// módulo normaliza/eserializa ese JSON con serde_json antes de entregarlo.

use serde::{Deserialize, Serialize};
use sqlx::{QueryBuilder, Row, SqlitePool};
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct FiltrosAuditoria {
    /// Nombre exacto de la tabla: trabajadores, recibos, periodos_nomina,
    /// incapacidades, vacaciones o audit_log.
    #[serde(default)]
    pub tabla: Option<String>,
    #[serde(default)]
    pub operacion: Option<String>,
    #[serde(default)]
    pub usuario: Option<String>,
    /// Fechas UTC en formato YYYY-MM-DD (se comparan con date(timestamp)).
    #[serde(default)]
    pub fecha_desde: Option<String>,
    #[serde(default)]
    pub fecha_hasta: Option<String>,
    /// Paginación del lado del servidor para la UI.
    #[serde(default)]
    pub limit: Option<i64>,
    #[serde(default)]
    pub offset: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RegistroAuditoria {
    pub id: i64,
    pub tabla: String,
    pub operacion: String,
    pub registro_id: i64,
    pub antes_json: Option<String>,
    pub despues_json: Option<String>,
    pub usuario: String,
    pub timestamp: String,
}

fn db_url(app: &AppHandle) -> String {
    let dir = app.path().app_data_dir().unwrap();
    format!("sqlite:{}", dir.join("nomina.db").display())
}

async fn pool(app: &AppHandle) -> Result<SqlitePool, String> {
    SqlitePool::connect(&db_url(app)).await.map_err(|e| e.to_string())
}

/// Ruta del config local de la app. Se usa el directorio de usuario del SO;
/// no se introduce una dependencia adicional solo para resolver $HOME.
fn config_path() -> Option<PathBuf> {
    #[cfg(windows)]
    let base = std::env::var_os("APPDATA").or_else(|| std::env::var_os("USERPROFILE"));
    #[cfg(not(windows))]
    let base = std::env::var_os("HOME");

    base.map(PathBuf::from)
        .map(|p| p.join(".nomina-mx").join("config.json"))
}

/// Usuario de la sesión: variable NOMINA_USUARIO → config.json → "system".
pub fn usuario_actual() -> String {
    if let Ok(valor) = std::env::var("NOMINA_USUARIO") {
        let valor = valor.trim();
        if !valor.is_empty() {
            return valor.to_string();
        }
    }
    if let Some(ruta) = config_path() {
        if let Ok(texto) = std::fs::read_to_string(ruta) {
            if let Ok(config) = serde_json::from_str::<serde_json::Value>(&texto) {
                for clave in ["NOMINA_USUARIO", "usuario"] {
                    if let Some(valor) = config.get(clave).and_then(|v| v.as_str()) {
                        if !valor.trim().is_empty() {
                            return valor.trim().to_string();
                        }
                    }
                }
            }
        }
    }
    "system".to_string()
}

/// Normaliza el JSON producido por json_object() con serde_json. Si el texto
/// no es JSON válido se conserva tal cual para no perder evidencia.
fn canonizar_json(raw: Option<&str>) -> Option<String> {
    raw.map(|texto| match serde_json::from_str::<serde_json::Value>(texto) {
        Ok(valor) => serde_json::to_string(&valor).unwrap_or_else(|_| texto.to_string()),
        Err(_) => texto.to_string(),
    })
}

/// Sincroniza el usuario activo en `audit_sesion`, de donde lo leen los
/// triggers al insertar en audit_log.
pub async fn sincronizar_usuario_pool(pool: &SqlitePool) -> Result<String, String> {
    let usuario = usuario_actual();
    sqlx::query(
        "INSERT INTO audit_sesion (id, usuario) VALUES (1, ?) \
         ON CONFLICT(id) DO UPDATE SET usuario = excluded.usuario",
    )
    .bind(&usuario)
    .execute(pool)
    .await
    .map_err(|e| format!("no se pudo sincronizar el usuario de auditoría: {}", e))?;
    Ok(usuario)
}

/// Permite a la UI fijar el usuario apenas arranca la app, antes de cualquier
/// alta/modificación, sin depender de que se abra primero la pantalla.
#[tauri::command]
pub async fn configurar_usuario_auditoria(app: AppHandle) -> Result<String, String> {
    let p = pool(&app).await?;
    sincronizar_usuario_pool(&p).await
}

/// Consulta con filtros opcionales. Los JSON se devuelven serializados por
/// serde_json (forma canónica) para que la UI pueda mostrarlos de forma estable.
pub async fn listar_auditoria_pool(
    pool: &SqlitePool,
    filtros: FiltrosAuditoria,
) -> Result<Vec<RegistroAuditoria>, String> {
    let mut qb = QueryBuilder::new(
        "SELECT id, tabla, operacion, registro_id, antes_json, despues_json, usuario, timestamp \
         FROM audit_log WHERE 1=1",
    );

    if let Some(v) = filtros.tabla.as_deref().filter(|s| !s.trim().is_empty()) {
        qb.push(" AND tabla = ").push_bind(v);
    }
    if let Some(v) = filtros
        .operacion
        .as_deref()
        .filter(|s| !s.trim().is_empty())
    {
        qb.push(" AND operacion = ").push_bind(v);
    }
    if let Some(v) = filtros.usuario.as_deref().filter(|s| !s.trim().is_empty()) {
        qb.push(" AND usuario = ").push_bind(v);
    }
    if let Some(v) = filtros
        .fecha_desde
        .as_deref()
        .filter(|s| !s.trim().is_empty())
    {
        qb.push(" AND date(timestamp) >= ").push_bind(v);
    }
    if let Some(v) = filtros
        .fecha_hasta
        .as_deref()
        .filter(|s| !s.trim().is_empty())
    {
        qb.push(" AND date(timestamp) <= ").push_bind(v);
    }

    qb.push(" ORDER BY id DESC LIMIT ").push_bind(filtros.limit.unwrap_or(500));
    qb.push(" OFFSET ").push_bind(filtros.offset.unwrap_or(0));

    let filas = qb
        .build()
        .fetch_all(pool)
        .await
        .map_err(|e| format!("error al consultar audit_log: {}", e))?;

    let mut salida = Vec::with_capacity(filas.len());
    for fila in filas {
        salida.push(RegistroAuditoria {
            id: fila.get("id"),
            tabla: fila.get("tabla"),
            operacion: fila.get("operacion"),
            registro_id: fila.get("registro_id"),
            antes_json: canonizar_json(fila.get::<Option<String>, _>("antes_json").as_deref()),
            despues_json: canonizar_json(fila.get::<Option<String>, _>("despues_json").as_deref()),
            usuario: fila.get("usuario"),
            timestamp: fila.get("timestamp"),
        });
    }
    Ok(salida)
}

#[tauri::command]
pub async fn listar_auditoria(
    app: AppHandle,
    filtros: FiltrosAuditoria,
) -> Result<Vec<RegistroAuditoria>, String> {
    let p = pool(&app).await?;
    // Toda consulta refresca el usuario activo; así NOMINA_USUARIO o el config
    // se reflejan incluso si la app se inició antes de configurarlos.
    sincronizar_usuario_pool(&p).await?;
    listar_auditoria_pool(&p, filtros).await
}
