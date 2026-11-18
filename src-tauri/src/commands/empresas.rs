// C4 — Multi-empresa: selección persistida en `config`.
//
// Fuentes: LISR art. 79 fracc. III (requisitos de deducción de nómina) y
// CFF art. 27 (contabilidad por establecimiento). El RFC único se refuerza
// con el índice UNIQUE creado en la migración 0004.
//
// La empresa activa NO se fía del cliente: se resuelve siempre en el servidor
// leyendo `config(clave='empresa_activa', valor=TEXT)`.

use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use tauri::AppHandle;
use tauri::Manager;

/// Clave de `config` donde se persiste el ID de la empresa activa (como texto).
pub const CLAVE_EMPRESA_ACTIVA: &str = "empresa_activa";

/// Fila de `empresas` en el orden canónico usado por todas las consultas.
type FilaEmpresa = (i64, String, String, String, String, String, f64, f64, String);

const COLUMNAS_EMPRESA: &str =
    "id, rfc, razon_social, regimen_fiscal, cp, estado_isn, tasa_isn, prima_riesgo_trabajo, registro_patronal_imss";

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Empresa {
    pub id: Option<i64>,
    pub rfc: String,
    pub razon_social: String,
    #[serde(default)]
    pub regimen_fiscal: String,
    #[serde(default)]
    pub cp: String,
    #[serde(default)]
    pub estado_isn: String,
    #[serde(default)]
    pub tasa_isn: f64,
    #[serde(default)]
    pub prima_riesgo_trabajo: f64,
    #[serde(default)]
    pub registro_patronal_imss: String,
    /// `true` cuando esta empresa es la seleccionada actualmente.
    #[serde(default)]
    pub activa: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NuevaEmpresa {
    pub rfc: String,
    pub razon_social: String,
    #[serde(default)]
    pub regimen_fiscal: String,
    #[serde(default)]
    pub cp: String,
    #[serde(default)]
    pub estado_isn: String,
    #[serde(default)]
    pub tasa_isn: f64,
    #[serde(default)]
    pub prima_riesgo_trabajo: f64,
    #[serde(default)]
    pub registro_patronal_imss: String,
}

fn db_url(app: &AppHandle) -> String {
    let dir = app.path().app_data_dir().unwrap();
    format!("sqlite:{}", dir.join("nomina.db").display())
}

async fn pool(app: &AppHandle) -> Result<SqlitePool, String> {
    SqlitePool::connect(&db_url(app)).await.map_err(|e| e.to_string())
}

fn fila_a_empresa(f: FilaEmpresa, activa_id: i64) -> Empresa {
    Empresa {
        id: Some(f.0),
        rfc: f.1,
        razon_social: f.2,
        regimen_fiscal: f.3,
        cp: f.4,
        estado_isn: f.5,
        tasa_isn: f.6,
        prima_riesgo_trabajo: f.7,
        registro_patronal_imss: f.8,
        activa: f.0 == activa_id,
    }
}

/// ID de la empresa activa. Si no existe configuración (BD anterior a C4)
/// se usa la Empresa Default (id = 1), que la migración garantiza.
pub async fn empresa_activa_pool(p: &SqlitePool) -> Result<i64, String> {
    let valor: Option<String> = sqlx::query_scalar("SELECT valor FROM config WHERE clave = ?")
        .bind(CLAVE_EMPRESA_ACTIVA)
        .fetch_optional(p)
        .await
        .map_err(|e| format!("error al leer la empresa activa: {}", e))?;

    match valor {
        Some(v) => v.trim().parse::<i64>().map_err(|_| {
            format!("valor inválido en config.{}: '{}'", CLAVE_EMPRESA_ACTIVA, v)
        }),
        None => Ok(1),
    }
}

/// Lista todas las empresas y marca cuál es la activa.
pub async fn listar_empresas_pool(p: &SqlitePool) -> Result<Vec<Empresa>, String> {
    let activa_id = empresa_activa_pool(p).await?;
    let sql = format!("SELECT {} FROM empresas ORDER BY id", COLUMNAS_EMPRESA);
    let filas = sqlx::query_as::<_, FilaEmpresa>(&sql)
        .fetch_all(p)
        .await
        .map_err(|e| format!("error al listar empresas: {}", e))?;
    Ok(filas
        .into_iter()
        .map(|f| fila_a_empresa(f, activa_id))
        .collect())
}

/// Devuelve la empresa activa. Si el ID configurado ya no existe, degrada a
/// la primera empresa disponible para no dejar la app sin contexto fiscal.
pub async fn obtener_empresa_activa_pool(p: &SqlitePool) -> Result<Option<Empresa>, String> {
    let activa_id = empresa_activa_pool(p).await?;
    let sql = format!("SELECT {} FROM empresas WHERE id = ?", COLUMNAS_EMPRESA);
    let fila = sqlx::query_as::<_, FilaEmpresa>(&sql)
        .bind(activa_id)
        .fetch_optional(p)
        .await
        .map_err(|e| format!("error al obtener la empresa activa: {}", e))?;

    if let Some(f) = fila {
        return Ok(Some(fila_a_empresa(f, activa_id)));
    }

    let sql = format!("SELECT {} FROM empresas ORDER BY id LIMIT 1", COLUMNAS_EMPRESA);
    let fila = sqlx::query_as::<_, FilaEmpresa>(&sql)
        .fetch_optional(p)
        .await
        .map_err(|e| format!("error al obtener la empresa activa: {}", e))?;
    Ok(fila.map(|f| {
        let id = f.0;
        fila_a_empresa(f, id)
    }))
}

/// Cambia la empresa activa y persiste el ID en `config`.
pub async fn cambiar_empresa_activa_pool(p: &SqlitePool, id: i64) -> Result<(), String> {
    let existe: Option<i64> = sqlx::query_scalar("SELECT id FROM empresas WHERE id = ?")
        .bind(id)
        .fetch_optional(p)
        .await
        .map_err(|e| format!("error al validar la empresa: {}", e))?;

    if existe.is_none() {
        return Err(format!("no existe una empresa con id {}", id));
    }

    sqlx::query(
        "INSERT INTO config (clave, valor) VALUES (?, ?) \
         ON CONFLICT(clave) DO UPDATE SET valor = excluded.valor",
    )
    .bind(CLAVE_EMPRESA_ACTIVA)
    .bind(id.to_string())
    .execute(p)
    .await
    .map_err(|e| format!("no se pudo persistir la empresa activa: {}", e))?;
    Ok(())
}

/// Crea una empresa validando RFC único. El índice UNIQUE de la BD es la
/// última línea de defensa; aquí se devuelve un mensaje claro para la UI.
pub async fn crear_empresa_pool(p: &SqlitePool, nueva: &NuevaEmpresa) -> Result<Empresa, String> {
    let rfc = nueva.rfc.trim();
    if rfc.is_empty() {
        return Err("el RFC de la empresa es obligatorio".to_string());
    }
    if nueva.razon_social.trim().is_empty() {
        return Err("la razón social de la empresa es obligatoria".to_string());
    }

    let duplicado: Option<i64> = sqlx::query_scalar("SELECT id FROM empresas WHERE rfc = ?")
        .bind(rfc)
        .fetch_optional(p)
        .await
        .map_err(|e| format!("error al validar el RFC: {}", e))?;
    if duplicado.is_some() {
        return Err(format!("ya existe una empresa con el RFC {}", rfc));
    }

    let res = sqlx::query(
        "INSERT INTO empresas (rfc, razon_social, regimen_fiscal, cp, estado_isn, \
         tasa_isn, prima_riesgo_trabajo, registro_patronal_imss) VALUES (?,?,?,?,?,?,?,?)",
    )
    .bind(rfc)
    .bind(nueva.razon_social.trim())
    .bind(&nueva.regimen_fiscal)
    .bind(&nueva.cp)
    .bind(&nueva.estado_isn)
    .bind(nueva.tasa_isn)
    .bind(nueva.prima_riesgo_trabajo)
    .bind(&nueva.registro_patronal_imss)
    .execute(p)
    .await
    .map_err(|e| format!("no se pudo crear la empresa: {}", e))?;

    let id = res.last_insert_rowid();
    Ok(Empresa {
        id: Some(id),
        rfc: rfc.to_string(),
        razon_social: nueva.razon_social.trim().to_string(),
        regimen_fiscal: nueva.regimen_fiscal.clone(),
        cp: nueva.cp.clone(),
        estado_isn: nueva.estado_isn.clone(),
        tasa_isn: nueva.tasa_isn,
        prima_riesgo_trabajo: nueva.prima_riesgo_trabajo,
        registro_patronal_imss: nueva.registro_patronal_imss.clone(),
        activa: false,
    })
}

#[tauri::command]
pub async fn listar_empresas(app: AppHandle) -> Result<Vec<Empresa>, String> {
    let p = pool(&app).await?;
    listar_empresas_pool(&p).await
}

#[tauri::command]
pub async fn cambiar_empresa_activa(app: AppHandle, id: i64) -> Result<(), String> {
    let p = pool(&app).await?;
    cambiar_empresa_activa_pool(&p, id).await
}

#[tauri::command]
pub async fn crear_empresa(app: AppHandle, nueva: NuevaEmpresa) -> Result<Empresa, String> {
    let p = pool(&app).await?;
    crear_empresa_pool(&p, &nueva).await
}
