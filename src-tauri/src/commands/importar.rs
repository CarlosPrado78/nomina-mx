// C2 — Importación masiva de trabajadores desde CSV.
//
// Fuentes: CFF art. 28 (conservación de información), LFT art. 2 y 25
// (relación de trabajo y salario) y catálogos del SAT (periodicidad de pago
// y tipo de contrato) usados también por el CFDI de Nómina 1.2.
//
// Reglas:
//   - UTF-8 con fallback a Latin-1 (Windows-1252).
//   - Columnas requeridas en el encabezado; falta alguna => Err (archivo).
//   - Cada fila se valida y se inserta en SU PROPIA transacción: una fila
//     inválida NO aborta el resto del archivo.
//   - `empresa_id` nunca viene del CSV: se toma de la empresa activa.

use crate::commands::empresas::obtener_empresa_activa_pool;
use regex::Regex;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use std::collections::HashMap;
use tauri::AppHandle;
use tauri::Manager;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ErrorFila {
    /// Número de fila del CSV (el encabezado es la fila 1).
    pub fila: usize,
    /// Nombre de la columna que falló (`fila` si el error es de estructura).
    pub columna: String,
    pub mensaje: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct ImportResult {
    pub insertados: usize,
    pub errores: Vec<ErrorFila>,
}

pub const COLUMNAS_REQUERIDAS: &[&str] = &[
    "nombre",
    "curp",
    "rfc",
    "nss",
    "fecha_ingreso",
    "salario_diario",
    "periodicidad_pago",
    "tipo_contrato",
];

pub const COLUMNAS_OPCIONALES: &[&str] = &[
    "num_empleado",
    "departamento",
    "puesto",
    "infonavit_activo",
    "infonavit_factor",
    "infonavit_tipo",
];

/// Catálogo SAT de PeriodicidadPago (códigos 01–10) en formato normalizado.
const PERIODICIDADES: &[&str] = &[
    "diaria", "semanal", "quincenal", "mensual", "bimestral", "trimestral",
    "semestral", "anual",
];

/// Catálogo SAT TipoContrato (01 indefinido, 02 temporal, 03 obra
/// determinada) más los alias de texto usados por la UI en español.
const TIPOS_CONTRATO: &[&str] = &[
    "01", "02", "03", "indefinido", "tiempo_indeterminado",
    "por_tiempo_indeterminado", "temporal", "tiempo_determinado",
    "por_tiempo_determinado", "obra_determinada", "por_obra_determinada",
];

fn db_url(app: &AppHandle) -> String {
    let dir = app.path().app_data_dir().unwrap();
    format!("sqlite:{}", dir.join("nomina.db").display())
}

async fn pool(app: &AppHandle) -> Result<SqlitePool, String> {
    SqlitePool::connect(&db_url(app)).await.map_err(|e| e.to_string())
}

/// Decodifica UTF-8; si los bytes no son UTF-8 válido aplica Windows-1252
/// (superconjunto de Latin-1, empleado por Excel en español).
fn decodificar(bytes: &[u8]) -> String {
    let texto = match std::str::from_utf8(bytes) {
        Ok(s) => s.to_string(),
        Err(_) => {
            let (cow, _, _) = encoding_rs::WINDOWS_1252.decode(bytes);
            cow.into_owned()
        }
    };
    texto.trim_start_matches('\u{feff}').to_string()
}

fn normalizar(valor: &str) -> String {
    valor.trim().to_lowercase().replace([' ', '-'], "_")
}

fn es_curp(valor: &str) -> bool {
    let v = valor.trim().to_uppercase();
    let patron = r"^[A-Z][AEIOUX][A-Z]{2}\d{6}[HM][A-Z]{5}[A-Z0-9]\d$";
    v.chars().count() == 18 && Regex::new(patron).map(|r| r.is_match(&v)).unwrap_or(false)
}

fn es_rfc(valor: &str) -> bool {
    let v = valor.trim().to_uppercase();
    let re = Regex::new(r"^[A-ZÑ&]{3,4}\d{6}[A-Z0-9]{3}$").unwrap();
    let n = v.chars().count();
    (n == 12 || n == 13) && re.is_match(&v)
}

fn es_nss(valor: &str) -> bool {
    let v = valor.trim();
    v.len() == 11 && v.chars().all(|c| c.is_ascii_digit())
}

fn es_fecha(valor: &str) -> bool {
    let v = valor.trim();
    let re = Regex::new(r"^\d{4}-\d{2}-\d{2}$").unwrap();
    re.is_match(v) && chrono::NaiveDate::parse_from_str(v, "%Y-%m-%d").is_ok()
}

fn es_booleano(valor: &str) -> Option<bool> {
    match normalizar(valor).as_str() {
        "" | "0" | "false" | "no" | "falso" => Some(false),
        "1" | "true" | "si" | "sí" | "verdadero" => Some(true),
        _ => None,
    }
}

/// Datos ya validados de una fila, listos para insertar.
struct FilaValida {
    nombre: String,
    curp: String,
    rfc: String,
    nss: String,
    num_empleado: String,
    fecha_ingreso: String,
    salario_diario: f64,
    periodicidad_pago: String,
    tipo_contrato: String,
    departamento: String,
    puesto: String,
    infonavit_activo: bool,
    infonavit_factor: f64,
    infonavit_tipo: String,
}

fn validar_fila(fila: usize, col: &HashMap<String, String>) -> Result<FilaValida, Vec<ErrorFila>> {
    let mut errores = Vec::new();
    let mut error = |columna: &str, mensaje: &str| {
        errores.push(ErrorFila {
            fila,
            columna: columna.to_string(),
            mensaje: mensaje.to_string(),
        });
    };

    let nombre = col.get("nombre").map(|s| s.trim().to_string()).unwrap_or_default();
    if nombre.is_empty() {
        error("nombre", "el nombre es obligatorio");
    }

    let curp = col.get("curp").map(|s| s.trim().to_uppercase()).unwrap_or_default();
    if !es_curp(&curp) {
        error("curp", "la CURP debe tener 18 caracteres con formato CURP válido");
    }

    let rfc = col.get("rfc").map(|s| s.trim().to_uppercase()).unwrap_or_default();
    if !es_rfc(&rfc) {
        error("rfc", "el RFC debe tener 12 o 13 caracteres con formato RFC válido");
    }

    let nss = col.get("nss").map(|s| s.trim().to_string()).unwrap_or_default();
    if !es_nss(&nss) {
        error("nss", "el NSS debe tener 11 dígitos");
    }

    let fecha_ingreso = col.get("fecha_ingreso").map(|s| s.trim().to_string()).unwrap_or_default();
    if !es_fecha(&fecha_ingreso) {
        error("fecha_ingreso", "la fecha debe tener formato YYYY-MM-DD y ser válida");
    }

    let salario_raw = col.get("salario_diario").map(|s| s.trim()).unwrap_or("");
    let salario_diario = salario_raw.parse::<f64>().unwrap_or(0.0);
    if !salario_diario.is_finite() || salario_diario <= 0.0 {
        error("salario_diario", "el salario diario debe ser un número mayor que cero");
    }

    let periodicidad_pago = normalizar(col.get("periodicidad_pago").map(|s| s.as_str()).unwrap_or(""));
    if !PERIODICIDADES.contains(&periodicidad_pago.as_str()) {
        error("periodicidad_pago", "valor fuera del catálogo SAT de periodicidad de pago");
    }

    let tipo_contrato = normalizar(col.get("tipo_contrato").map(|s| s.as_str()).unwrap_or(""));
    if !TIPOS_CONTRATO.contains(&tipo_contrato.as_str()) {
        error("tipo_contrato", "valor fuera del catálogo SAT de tipo de contrato");
    }

    let factor_raw = col.get("infonavit_factor").map(|s| s.trim()).unwrap_or("");
    let infonavit_factor = if factor_raw.is_empty() {
        0.0
    } else {
        factor_raw.parse::<f64>().unwrap_or(f64::NAN)
    };
    if !infonavit_factor.is_finite() || infonavit_factor < 0.0 {
        error("infonavit_factor", "el factor INFONAVIT debe ser un número >= 0");
    }

    let activo_raw = col.get("infonavit_activo").map(|s| s.as_str()).unwrap_or("");
    let infonavit_activo = match es_booleano(activo_raw) {
        Some(v) => v,
        None => {
            error("infonavit_activo", "use 1/0, true/false o si/no");
            false
        }
    };

    if !errores.is_empty() {
        return Err(errores);
    }

    Ok(FilaValida {
        nombre,
        curp,
        rfc,
        nss,
        num_empleado: col.get("num_empleado").map(|s| s.trim().to_string()).unwrap_or_default(),
        fecha_ingreso,
        salario_diario,
        periodicidad_pago,
        tipo_contrato,
        departamento: col.get("departamento").map(|s| s.trim().to_string()).unwrap_or_default(),
        puesto: col.get("puesto").map(|s| s.trim().to_string()).unwrap_or_default(),
        infonavit_activo,
        infonavit_factor,
        infonavit_tipo: col.get("infonavit_tipo").map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty()).unwrap_or_else(|| "CUOTA_FIJA_VSM".to_string()),
    })
}

/// Inserta una fila dentro de su propia transacción (atomicidad por fila).
async fn insertar_fila(p: &SqlitePool, empresa_id: i64, f: &FilaValida) -> Result<(), String> {
    let mut tx = p.begin().await.map_err(|e| e.to_string())?;
    sqlx::query(
        "INSERT INTO trabajadores (
           empresa_id, nombre, curp, rfc, nss, num_empleado, fecha_nacimiento,
           estado_civil, fecha_ingreso, tipo_contrato, departamento, puesto,
           salario_diario, periodicidad_pago, jornada, horas_semanales,
           infonavit_activo, infonavit_num_credito, infonavit_factor,
           infonavit_tipo_descuento, fecha_baja, tipo_baja
         ) VALUES (?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?)",
    )
    .bind(empresa_id)
    .bind(&f.nombre)
    .bind(&f.curp)
    .bind(&f.rfc)
    .bind(&f.nss)
    .bind(&f.num_empleado)
    .bind("")
    .bind("")
    .bind(&f.fecha_ingreso)
    .bind(&f.tipo_contrato)
    .bind(&f.departamento)
    .bind(&f.puesto)
    .bind(f.salario_diario)
    .bind(&f.periodicidad_pago)
    .bind("diurna")
    .bind(40.0_f64)
    .bind(f.infonavit_activo as i64)
    .bind(Option::<String>::None)
    .bind(f.infonavit_factor)
    .bind(&f.infonavit_tipo)
    .bind(Option::<String>::None)
    .bind(Option::<String>::None)
    .execute(&mut *tx)
    .await
    .map_err(|e| e.to_string())?;
    tx.commit().await.map_err(|e| e.to_string())
}

/// Importa un CSV. Errores de archivo/encabezado => Err; errores de fila =>
/// `ImportResult.errores`, continuando con las demás filas.
pub async fn importar_trabajadores_csv_pool(
    p: &SqlitePool,
    ruta: &str,
) -> Result<ImportResult, String> {
    let bytes = std::fs::read(ruta)
        .map_err(|e| format!("no se pudo leer el archivo '{}': {}", ruta, e))?;
    let texto = decodificar(&bytes);

    let mut reader = csv::ReaderBuilder::new()
        .has_headers(true)
        .trim(csv::Trim::All)
        .from_reader(texto.as_bytes());

    let cabeceras = reader
        .headers()
        .map_err(|e| format!("no se pudo leer el encabezado del CSV: {}", e))?;
    let indice: HashMap<String, usize> = cabeceras
        .iter()
        .enumerate()
        .map(|(i, h)| (normalizar(h), i))
        .collect();

    for requerida in COLUMNAS_REQUERIDAS {
        if !indice.contains_key(*requerida) {
            return Err(format!("falta la columna requerida: {}", requerida));
        }
    }

    let empresa_activa = obtener_empresa_activa_pool(p).await?
        .ok_or_else(|| "no hay una empresa activa; configure una antes de importar".to_string())?;
    let empresa_id = empresa_activa.id
        .ok_or_else(|| "la empresa activa no tiene identificador".to_string())?;
    let mut resultado = ImportResult::default();

    for (i, registro) in reader.records().enumerate() {
        let fila = i + 2; // encabezado = fila 1
        let registro = match registro {
            Ok(r) => r,
            Err(e) => {
                resultado.errores.push(ErrorFila {
                    fila,
                    columna: "fila".to_string(),
                    mensaje: format!("registro mal formado: {}", e),
                });
                continue;
            }
        };

        let col: HashMap<String, String> = indice
            .iter()
            .filter_map(|(nombre, pos)| registro.get(*pos).map(|v| (nombre.clone(), v.to_string())))
            .collect();

        let datos = match validar_fila(fila, &col) {
            Ok(d) => d,
            Err(errores) => {
                resultado.errores.extend(errores);
                continue;
            }
        };

        match insertar_fila(p, empresa_id, &datos).await {
            Ok(()) => resultado.insertados += 1,
            Err(e) => resultado.errores.push(ErrorFila {
                fila,
                columna: "base_de_datos".to_string(),
                mensaje: e,
            }),
        }
    }

    Ok(resultado)
}

#[tauri::command]
pub async fn importar_trabajadores_csv(app: AppHandle, ruta: String) -> Result<ImportResult, String> {
    let p = pool(&app).await?;
    importar_trabajadores_csv_pool(&p, &ruta).await
}
