// Módulo de base de datos: ejecuta las migraciones propias de NominaMX.
//
// Antes se usaban las migraciones de tauri_plugin_sql, que guardan un checksum
// de cada SQL aplicado. Eso impide corregir un seed ya instalado: si el SQL de
// una migración aplicada cambia, el plugin aborta el arranque con
// "migration 2 was previously applied but has been modified". Como el perfil
// release usa panic = "abort", ese error cerraba la aplicación sin interfaz.
//
// Ahora las migraciones son propias: se registran por versión y solo se
// ejecutan las que faltan. Los cambios de datos se agregan como una migración
// nueva (versión 6), nunca editando una ya aplicada.

use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::Row;
use std::path::Path;

struct Migration {
    version: i64,
    description: &'static str,
    sql: &'static str,
}

const MIGRATIONS: &[Migration] = &[
    Migration {
        version: 1,
        description: "init",
        sql: include_str!("../../migrations/0001_init.sql"),
    },
    Migration {
        version: 2,
        description: "seed fiscal 2026",
        sql: include_str!("../../migrations/0002_seed_fiscal_2026.sql"),
    },
    Migration {
        version: 3,
        description: "bloque B: parametros de horas extra",
        sql: include_str!("../../migrations/0003_horas_extra.sql"),
    },
    Migration {
        version: 4,
        description: "C3: log de auditoria inmutable",
        sql: include_str!("../../migrations/0003_audit_log.sql"),
    },
    Migration {
        version: 5,
        description: "C4: multi-empresa con seleccion persistida",
        sql: include_str!("../../migrations/0004_multi_empresa.sql"),
    },
    Migration {
        version: 6,
        description: "seed fiscal 2026 actualizado (UMA y subsidio por vigencia)",
        sql: include_str!("../../migrations/0005_actualiza_seed_2026.sql"),
    },
];

/// Aplica las migraciones pendientes sobre `app_dir/nomina.db`.
///
/// Es idempotente: ejecutarla dos veces no duplica datos ni falla. Las bases
/// creadas por versiones anteriores tenían `_sqlx_migrations`; su historial se
/// traslada a `_nomina_migrations` para no repetir migraciones ya aplicadas.
pub async fn run_migrations_async(app_dir: &Path) -> Result<(), String> {
    std::fs::create_dir_all(app_dir)
        .map_err(|e| format!("no se pudo crear {}: {e}", app_dir.display()))?;
    let db_path = app_dir.join("nomina.db");

    let opciones = SqliteConnectOptions::new()
        .filename(&db_path)
        .create_if_missing(true);
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(opciones)
        .await
        .map_err(|e| format!("no se pudo abrir la BD: {e}"))?;

    sqlx::raw_sql(
        "CREATE TABLE IF NOT EXISTS _nomina_migrations (\
         version INTEGER PRIMARY KEY,\
         description TEXT NOT NULL,\
         installed_on TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP\
         );",
    )
    .execute(&pool)
    .await
    .map_err(|e| format!("no se pudo crear la tabla de migraciones: {e}"))?;

    let legacy: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='_sqlx_migrations'",
    )
    .fetch_one(&pool)
    .await
    .map_err(|e| format!("no se pudo revisar migraciones anteriores: {e}"))?;
    if legacy > 0 {
        let filas = sqlx::query("SELECT version, description FROM _sqlx_migrations WHERE success = 1")
            .fetch_all(&pool)
            .await
            .map_err(|e| format!("no se pudo leer migraciones anteriores: {e}"))?;
        for fila in filas {
            let version: i64 = fila.get("version");
            let description: String = fila.get("description");
            sqlx::query(
                "INSERT OR IGNORE INTO _nomina_migrations (version, description) VALUES (?, ?)",
            )
            .bind(version)
            .bind(description)
            .execute(&pool)
            .await
            .map_err(|e| format!("no se pudo migrar el historial: {e}"))?;
        }
    }

    for migracion in MIGRATIONS {
        let aplicada: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM _nomina_migrations WHERE version = ?",
        )
        .bind(migracion.version)
        .fetch_one(&pool)
        .await
        .map_err(|e| format!("no se pudo verificar la migración {}: {e}", migracion.version))?;
        if aplicada > 0 {
            continue;
        }

        sqlx::raw_sql(migracion.sql)
            .execute(&pool)
            .await
            .map_err(|e| format!("migración {} falló: {e}", migracion.version))?;
        sqlx::query("INSERT INTO _nomina_migrations (version, description) VALUES (?, ?)")
            .bind(migracion.version)
            .bind(migracion.description)
            .execute(&pool)
            .await
            .map_err(|e| format!("no se pudo registrar la migración {}: {e}", migracion.version))?;
    }

    pool.close().await;
    Ok(())
}

/// Versión síncrona usada durante el `setup` de Tauri.
pub fn run_migrations(app_dir: &Path) -> Result<(), String> {
    tauri::async_runtime::block_on(run_migrations_async(app_dir))
}
