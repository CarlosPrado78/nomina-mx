// C1 — Backup / restore de la base de datos SQLite.
//
// Referencia: art. 28 del CFF (conservación de documentación y registros) y
// guías del SAT sobre conservación de información contable. No es una
// obligación fiscal directa de "hacer backup", sino una buena práctica
// operativa para poder conservar y reproducir la información.
//
// sqlx 0.8 no expone la API `backup()` de SQLite, por lo que se usa la
// sentencia SQL `VACUUM INTO`, que es la forma SQL del mecanismo online
// backup de SQLite (equivalente a `.backup` de la CLI). Produce un
// archivo consistente sin bloquear la BD activa.

use chrono::Local;
use serde::{Deserialize, Serialize};
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::SqlitePool;
use std::fs::{self, OpenOptions};
use std::path::{Path, PathBuf};
use tauri::{AppHandle, Manager};

const CABECERA_SQLITE: &[u8] = b"SQLite format 3\0";

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ResumenBackup {
    pub nombre: String,
    pub ruta: String,
    pub fecha: String,
    pub tamano: u64,
}

fn db_path(app: &AppHandle) -> PathBuf {
    app.path().app_data_dir().unwrap().join("nomina.db")
}

fn backup_dir(app: &AppHandle) -> PathBuf {
    app.path().app_data_dir().unwrap().join("backups")
}

/// Valida que el directorio destino exista y sea escribible.
/// En Windows los bits de permisos no son confiables, se prueba creando
/// y borrando un archivo temporal.
pub fn validar_directorio_destino(destino: &Path) -> Result<(), String> {
    if !destino.exists() {
        return Err(format!("el directorio destino no existe: {}", destino.display()));
    }
    if !destino.is_dir() {
        return Err(format!("la ruta destino no es un directorio: {}", destino.display()));
    }
    let prueba = destino.join(format!(".__nomina_write_test_{}", std::process::id()));
    OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&prueba)
        .map_err(|e| format!("directorio destino no escribible: {}", e))?;
    fs::remove_file(&prueba).map_err(|e| e.to_string())?;
    Ok(())
}

/// Nombre con timestamp: nomina_backup_YYYYMMDD_HHMMSS.db.
/// Si ya existe (dos backups en el mismo segundo), agrega _1, _2, ...
pub fn nombre_backup_unico(destino: &Path, stamp: &str) -> PathBuf {
    let base = destino.join(format!("nomina_backup_{}.db", stamp));
    if !base.exists() {
        return base;
    }
    for i in 1..10_000 {
        let candidato = destino.join(format!("nomina_backup_{}_{}.db", stamp, i));
        if !candidato.exists() {
            return candidato;
        }
    }
    destino.join(format!("nomina_backup_{}_{}.db", stamp, Local::now().timestamp_nanos_opt().unwrap_or(0)))
}

async fn abrir_pool(path: &Path, crear: bool) -> Result<SqlitePool, String> {
    SqlitePoolOptions::new()
        .connect_with(
            SqliteConnectOptions::new()
                .filename(path)
                .create_if_missing(crear),
        )
        .await
        .map_err(|e| e.to_string())
}

async fn verificar_integridad(path: &Path) -> Result<(), String> {
    let pool = abrir_pool(path, false).await?;
    let resultado: String = sqlx::query_scalar("PRAGMA integrity_check")
        .fetch_one(&pool)
        .await
        .map_err(|e| e.to_string())?;
    pool.close().await;
    if resultado.eq_ignore_ascii_case("ok") {
        Ok(())
    } else {
        Err(format!("PRAGMA integrity_check devolvió: {}", resultado))
    }
}

/// Valida la cabecera SQLite antes de intentar abrir el archivo.
pub fn validar_cabecera_sqlite(path: &Path) -> Result<(), String> {
    let mut archivo = fs::File::open(path)
        .map_err(|e| format!("no se pudo abrir el archivo origen: {}", e))?;
    let mut cabecera = [0u8; 16];
    use std::io::Read;
    archivo
        .read_exact(&mut cabecera)
        .map_err(|e| format!("archivo sin cabecera SQLite: {}", e))?;
    if cabecera != CABECERA_SQLITE {
        return Err("archivo sin cabecera SQLite: se esperaba 'SQLite format 3\\0'".to_string());
    }
    Ok(())
}

/// Crea un respaldo consistente de `db_path` dentro de `destino`.
pub async fn crear_backup(db_path: &Path, destino: &Path) -> Result<PathBuf, String> {
    let stamp = Local::now().format("%Y%m%d_%H%M%S").to_string();
    crear_backup_con_stamp(db_path, destino, &stamp).await
}

/// Variante con timestamp explícito, usada por los tests de no sobrescritura.
pub async fn crear_backup_con_stamp(
    db_path: &Path,
    destino: &Path,
    stamp: &str,
) -> Result<PathBuf, String> {
    validar_directorio_destino(destino)?;
    if !db_path.exists() {
        return Err(format!("la BD activa no existe: {}", db_path.display()));
    }
    let destino_final = nombre_backup_unico(destino, stamp);
    let sql = format!(
        "VACUUM INTO '{}'",
        destino_final.to_string_lossy().replace('\'', "''")
    );
    let pool = abrir_pool(db_path, false).await?;
    sqlx::query(&sql)
        .execute(&pool)
        .await
        .map_err(|e| {
            let _ = fs::remove_file(&destino_final);
            format!("VACUUM INTO falló: {}", e)
        })?;
    pool.close().await;
    verificar_integridad(&destino_final).await?;
    Ok(destino_final)
}

/// Restaura un respaldo validado. Antes copia la BD activa a `backup_dir`
/// para que la operación sea reversible.
pub async fn restaurar_backup(
    db_activa: &Path,
    origen: &Path,
    backup_dir: &Path,
) -> Result<PathBuf, String> {
    validar_cabecera_sqlite(origen)?;
    verificar_integridad(origen).await?;

    // Respaldo automático previo (nunca se pierde el estado anterior).
    let respaldo_previo = crear_backup(db_activa, backup_dir).await?;

    // Cerrar cualquier pool abierto por esta operación antes de reemplazar.
    let pool = abrir_pool(db_activa, false).await?;
    sqlx::query("PRAGMA wal_checkpoint(TRUNCATE)")
        .execute(&pool)
        .await
        .ok();
    pool.close().await;

    fs::copy(origen, db_activa)
        .map_err(|e| format!("no se pudo reemplazar la BD activa: {}", e))?;
    verificar_integridad(db_activa).await?;
    Ok(respaldo_previo)
}

/// Lista los respaldos con fecha (nombre) y tamaño en bytes.
pub fn listar_respaldos(destino: &Path) -> Result<Vec<ResumenBackup>, String> {
    validar_directorio_destino(destino)?;
    let mut items = Vec::new();
    for entrada in fs::read_dir(destino).map_err(|e| e.to_string())? {
        let entrada = entrada.map_err(|e| e.to_string())?;
        let ruta = entrada.path();
        if !ruta.is_file() {
            continue;
        }
        let nombre = entrada.file_name().to_string_lossy().to_string();
        if !nombre.starts_with("nomina_backup_") || !nombre.ends_with(".db") {
            continue;
        }
        let meta = entrada.metadata().map_err(|e| e.to_string())?;
        items.push(ResumenBackup {
            nombre: nombre.clone(),
            ruta: ruta.to_string_lossy().to_string(),
            fecha: nombre
                .trim_start_matches("nomina_backup_")
                .trim_end_matches(".db")
                .to_string(),
            tamano: meta.len(),
        });
    }
    items.sort_by(|a, b| a.nombre.cmp(&b.nombre));
    Ok(items)
}

#[tauri::command]
pub async fn backup_db(app: AppHandle, destino: String) -> Result<String, String> {
    let dir = if destino.trim().is_empty() {
        let dir = backup_dir(&app);
        fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
        dir
    } else {
        // El destino proporcionado por el usuario DEBE existir y ser escribible;
        // no se crea implícitamente para poder reportar un error claro.
        PathBuf::from(destino)
    };
    validar_directorio_destino(&dir)?;
    let ruta = crear_backup(&db_path(&app), &dir).await?;
    Ok(ruta.to_string_lossy().to_string())
}

#[tauri::command]
pub async fn restore_db(app: AppHandle, origen: String) -> Result<String, String> {
    let dir = backup_dir(&app);
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let previo = restaurar_backup(&db_path(&app), Path::new(&origen), &dir).await?;
    Ok(previo.to_string_lossy().to_string())
}

#[tauri::command]
pub async fn listar_backups(app: AppHandle) -> Result<Vec<ResumenBackup>, String> {
    let dir = backup_dir(&app);
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    listar_respaldos(&dir)
}
