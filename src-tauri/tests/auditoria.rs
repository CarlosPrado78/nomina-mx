// C3 — Evidencia del log de auditoría inmutable.
//
// Fuentes: CFF art. 28 (conservación de información y registros) y LSS art. 15
// (registro de movimientos afiliatorios). Cada caso verifica el comportamiento
// esperado de los triggers y del comando de consulta. `timestamp` se almacena
// con CURRENT_TIMESTAMP (UTC en SQLite).

use nomina_mx_lib::commands::auditoria::{
    listar_auditoria_pool, sincronizar_usuario_pool, FiltrosAuditoria,
};
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::{Row, SqlitePool};
use std::fs;
use std::path::PathBuf;

struct Escenario {
    raiz: PathBuf,
    pool: SqlitePool,
}

impl Drop for Escenario {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.raiz);
    }
}

async fn db_nuevo(nombre: &str) -> Escenario {
    let nano = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let raiz = std::env::temp_dir().join(format!(
        "nomina_c3_{}_{}_{}",
        nombre,
        std::process::id(),
        nano
    ));
    if raiz.exists() {
        fs::remove_dir_all(&raiz).unwrap();
    }
    fs::create_dir_all(&raiz).unwrap();
    let ruta_db = raiz.join("nomina.db");

    let pool = SqlitePoolOptions::new()
        .connect_with(
            SqliteConnectOptions::new()
                .filename(&ruta_db)
                .create_if_missing(true),
        )
        .await
        .unwrap();

    // Se aplican las cuatro migraciones reales del proyecto, en el mismo
    // orden en que las registra lib.rs (audit_log es la versión 4).
    for sql in [
        include_str!("../migrations/0001_init.sql"),
        include_str!("../migrations/0002_seed_fiscal_2026.sql"),
        include_str!("../migrations/0003_horas_extra.sql"),
        include_str!("../migrations/0003_audit_log.sql"),
    ] {
        sqlx::raw_sql(sql).execute(&pool).await.unwrap();
    }

    // sqlx activa FOREIGN KEY; se crea la empresa referenced por los fixtures.
    sqlx::query(
        "INSERT INTO empresas (id, rfc, razon_social) VALUES (1, 'XAXX010101000', 'Empresa Demo')",
    )
    .execute(&pool)
    .await
    .unwrap();

    // Usuario de sesión controlado para que el assertion sea determinista.
    sincronizar_usuario_pool(&pool).await.unwrap();
    sqlx::query("UPDATE audit_sesion SET usuario = 'tester' WHERE id = 1")
        .execute(&pool)
        .await
        .unwrap();

    Escenario { raiz, pool }
}

async fn nuevo_trabajador(pool: &SqlitePool, nombre: &str) -> i64 {
    sqlx::query("INSERT INTO trabajadores (empresa_id, nombre, salario_diario) VALUES (1, ?, 400)")
        .bind(nombre)
        .execute(pool)
        .await
        .unwrap()
        .last_insert_rowid()
}

async fn nuevo_periodo(pool: &SqlitePool) -> i64 {
    sqlx::query(
        "INSERT INTO periodos_nomina (empresa_id, tipo, fecha_inicio, fecha_fin, fecha_pago, estado, dias) \
         VALUES (1, 'mensual', '2026-01-01', '2026-01-31', '2026-01-31', 'cerrado', 31)",
    )
    .execute(pool)
    .await
    .unwrap()
    .last_insert_rowid()
}

#[tokio::test]
async fn trigger_insert_trabajador_crea_audit_log() {
    // Esperado: INSERT en trabajadores → 1 fila con operacion="INSERT",
    // antes_json=NULL, despues_json con los datos del registro y usuario activo.
    // Fuente: CFF art. 28 (conservación de registros).
    let s = db_nuevo("insert").await;
    let id = nuevo_trabajador(&s.pool, "Juan Pérez").await;

    let fila = sqlx::query(
        "SELECT operacion, antes_json, despues_json, usuario FROM audit_log \
         WHERE tabla = 'trabajadores' AND registro_id = ?",
    )
    .bind(id)
    .fetch_one(&s.pool)
    .await
    .unwrap();

    assert_eq!(fila.get::<String, _>("operacion"), "INSERT");
    assert!(fila.get::<Option<String>, _>("antes_json").is_none());
    let despues: serde_json::Value =
        serde_json::from_str(&fila.get::<String, _>("despues_json")).unwrap();
    assert_eq!(despues["nombre"], "Juan Pérez");
    assert_eq!(despues["salario_diario"], 400.0);
    assert_eq!(fila.get::<String, _>("usuario"), "tester");
}

#[tokio::test]
async fn trigger_update_trabajador_guarda_antes_y_despues() {
    // Esperado: UPDATE → 1 fila con ambos JSON distintos.
    // Manual: nombre "Juan" → "Juan Actualizado" y salario 400 → 500.
    // Fuente: CFF art. 28; LSS art. 15 (movimientos afiliatorios).
    let s = db_nuevo("update").await;
    let id = nuevo_trabajador(&s.pool, "Juan").await;

    sqlx::query("UPDATE trabajadores SET nombre = 'Juan Actualizado', salario_diario = 500 WHERE id = ?")
        .bind(id)
        .execute(&s.pool)
        .await
        .unwrap();

    let fila = sqlx::query(
        "SELECT antes_json, despues_json FROM audit_log \
         WHERE tabla = 'trabajadores' AND operacion = 'UPDATE' AND registro_id = ?",
    )
    .bind(id)
    .fetch_one(&s.pool)
    .await
    .unwrap();

    let antes_raw = fila.get::<String, _>("antes_json");
    let despues_raw = fila.get::<String, _>("despues_json");
    assert_ne!(antes_raw, despues_raw);

    let antes: serde_json::Value = serde_json::from_str(&antes_raw).unwrap();
    let despues: serde_json::Value = serde_json::from_str(&despues_raw).unwrap();
    assert_eq!(antes["nombre"], "Juan");
    assert_eq!(antes["salario_diario"], 400.0);
    assert_eq!(despues["nombre"], "Juan Actualizado");
    assert_eq!(despues["salario_diario"], 500.0);
}

#[tokio::test]
async fn trigger_delete_trabajador_guarda_registro() {
    // Esperado: DELETE → 1 fila con antes_json del registro borrado y
    // despues_json=NULL. Fuente: CFF art. 28 (conservación de evidencia).
    let s = db_nuevo("delete").await;
    let id = nuevo_trabajador(&s.pool, "María").await;

    sqlx::query("DELETE FROM trabajadores WHERE id = ?")
        .bind(id)
        .execute(&s.pool)
        .await
        .unwrap();

    let fila = sqlx::query(
        "SELECT operacion, antes_json, despues_json FROM audit_log \
         WHERE tabla = 'trabajadores' AND operacion = 'DELETE' AND registro_id = ?",
    )
    .bind(id)
    .fetch_one(&s.pool)
    .await
    .unwrap();

    let antes: serde_json::Value =
        serde_json::from_str(&fila.get::<String, _>("antes_json")).unwrap();
    assert_eq!(antes["nombre"], "María");
    assert!(fila.get::<Option<String>, _>("despues_json").is_none());
}

#[tokio::test]
async fn audit_log_no_se_puede_actualizar() {
    // Esperado: UPDATE y DELETE sobre audit_log fallan con RAISE(ABORT).
    // Regla: los registros de auditoría son inmutables (solo INSERT).
    let s = db_nuevo("inmutable").await;
    let id = nuevo_trabajador(&s.pool, "Ana").await;
    let audit_id: i64 = sqlx::query_scalar("SELECT id FROM audit_log LIMIT 1")
        .fetch_one(&s.pool)
        .await
        .unwrap();

    let err_update = sqlx::query("UPDATE audit_log SET usuario = 'otro' WHERE id = ?")
        .bind(audit_id)
        .execute(&s.pool)
        .await
        .unwrap_err();
    assert!(
        err_update.to_string().contains("inmutable"),
        "{}",
        err_update
    );

    let err_delete = sqlx::query("DELETE FROM audit_log WHERE id = ?")
        .bind(audit_id)
        .execute(&s.pool)
        .await
        .unwrap_err();
    assert!(
        err_delete.to_string().contains("inmutable"),
        "{}",
        err_delete
    );

    let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM audit_log")
        .fetch_one(&s.pool)
        .await
        .unwrap();
    assert_eq!(total, 1, "el intento de borrado no debió afectar la tabla");
    let _ = id;
}

#[tokio::test]
async fn listar_auditoria_filtra_por_tabla() {
    // Esperado: 3 cambios en trabajadores (1 INSERT + 2 UPDATE) y 2 INSERT en
    // recibos. El filtro por tabla='trabajadores' devuelve exactamente 3.
    // Además se comprueba que los JSON devueltos son parseables por serde_json.
    let s = db_nuevo("filtro").await;
    let trabajador = nuevo_trabajador(&s.pool, "Luis").await;
    sqlx::query("UPDATE trabajadores SET salario_diario = 450 WHERE id = ?")
        .bind(trabajador)
        .execute(&s.pool)
        .await
        .unwrap();
    sqlx::query("UPDATE trabajadores SET puesto = 'Analista' WHERE id = ?")
        .bind(trabajador)
        .execute(&s.pool)
        .await
        .unwrap();

    let periodo = nuevo_periodo(&s.pool).await;
    for neto in [1000.0, 1100.0] {
        sqlx::query("INSERT INTO recibos (periodo_id, trabajador_id, neto) VALUES (?, ?, ?)")
            .bind(periodo)
            .bind(trabajador)
            .bind(neto)
            .execute(&s.pool)
            .await
            .unwrap();
    }

    let filas = listar_auditoria_pool(
        &s.pool,
        FiltrosAuditoria {
            tabla: Some("trabajadores".into()),
            ..Default::default()
        },
    )
    .await
    .unwrap();

    assert_eq!(filas.len(), 3, "filtro por tabla trabajadores");
    assert!(filas.iter().all(|f| f.tabla == "trabajadores"));
    for fila in &filas {
        if let Some(json) = &fila.despues_json {
            serde_json::from_str::<serde_json::Value>(json).unwrap();
        }
    }

    let recibos = listar_auditoria_pool(
        &s.pool,
        FiltrosAuditoria {
            tabla: Some("recibos".into()),
            ..Default::default()
        },
    )
    .await
    .unwrap();
    assert_eq!(recibos.len(), 2, "filtro por tabla recibos");
}
