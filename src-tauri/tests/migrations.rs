// Regresión del crash de macOS: el runner propio de migraciones debe abrir
// bases nuevas y bases creadas por tauri_plugin_sql sin validar checksums,
// aplicando el seed 2026 correcto sin borrar datos de negocio.

use nomina_mx_lib::db::run_migrations_async;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::{Row, SqlitePool};
use std::fs;
use std::path::PathBuf;

struct Escenario {
    raiz: PathBuf,
}

impl Drop for Escenario {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.raiz);
    }
}

fn raiz(nombre: &str) -> Escenario {
    let nano = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let raiz = std::env::temp_dir().join(format!(
        "nomina_migrations_{}_{}_{}",
        nombre,
        std::process::id(),
        nano
    ));
    if raiz.exists() {
        fs::remove_dir_all(&raiz).unwrap();
    }
    fs::create_dir_all(&raiz).unwrap();
    Escenario { raiz }
}

async fn conectar(raiz: &PathBuf) -> SqlitePool {
    SqlitePoolOptions::new()
        .connect_with(
            SqliteConnectOptions::new()
                .filename(raiz.join("nomina.db"))
                .create_if_missing(true),
        )
        .await
        .unwrap()
}

async fn versiones(pool: &SqlitePool) -> Vec<i64> {
    sqlx::query_scalar("SELECT version FROM _nomina_migrations ORDER BY version")
        .fetch_all(pool)
        .await
        .unwrap()
}

async fn filas_seed(pool: &SqlitePool, tipo: &str) -> Vec<(String, String)> {
    let filas = sqlx::query(
        "SELECT vigencia_inicio, datos_json FROM tablas_fiscales \
         WHERE anio = 2026 AND tipo = ? ORDER BY vigencia_inicio",
    )
    .bind(tipo)
    .fetch_all(pool)
    .await
    .unwrap();
    filas
        .into_iter()
        .map(|f| (f.get("vigencia_inicio"), f.get("datos_json")))
        .collect()
}

#[tokio::test]
async fn bd_nueva_aplica_seis_migraciones_y_seed_correcto() {
    let escenario = raiz("nueva");
    run_migrations_async(&escenario.raiz).await.unwrap();

    let pool = conectar(&escenario.raiz).await;
    assert_eq!(versiones(&pool).await, vec![1, 2, 3, 4, 5, 6]);

    let uma = filas_seed(&pool, "uma").await;
    assert_eq!(uma.len(), 2);
    assert_eq!(uma[0].0, "2026-01-01");
    assert!(uma[0].1.contains("113.14"), "{}", uma[0].1);
    assert_eq!(uma[1].0, "2026-02-01");
    assert!(uma[1].1.contains("117.31"), "{}", uma[1].1);

    let subsidio = filas_seed(&pool, "subsidio_empleo").await;
    assert_eq!(subsidio.len(), 2);
    assert_eq!(subsidio[0].0, "2026-01-01");
    assert!(subsidio[0].1.contains("536.21"), "{}", subsidio[0].1);
    assert_eq!(subsidio[1].0, "2026-02-01");
    assert!(subsidio[1].1.contains("535.65"), "{}", subsidio[1].1);
    pool.close().await;

    // El runner se vuelve a ejecutar en cada arranque: no debe duplicar nada.
    run_migrations_async(&escenario.raiz).await.unwrap();
    let pool = conectar(&escenario.raiz).await;
    assert_eq!(versiones(&pool).await, vec![1, 2, 3, 4, 5, 6]);
    assert_eq!(filas_seed(&pool, "uma").await.len(), 2);
    assert_eq!(filas_seed(&pool, "subsidio_empleo").await.len(), 2);
}

#[tokio::test]
async fn bd_legacy_actualiza_seed_y_conserva_datos_de_negocio() {
    let escenario = raiz("legacy");
    let pool = conectar(&escenario.raiz).await;

    // Estado real del equipo que crasheó: migraciones 1 y 2 instaladas por
    // tauri_plugin_sql con checksum viejo, y seed 2026 de una sola vigencia.
    sqlx::raw_sql(include_str!("../migrations/0001_init.sql"))
        .execute(&pool)
        .await
        .unwrap();
    sqlx::raw_sql(
        "CREATE TABLE _sqlx_migrations (\
         version BIGINT PRIMARY KEY,\
         description TEXT NOT NULL,\
         installed_on TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,\
         success BOOLEAN NOT NULL,\
         checksum BLOB NOT NULL\
         );\
         INSERT INTO _sqlx_migrations (version, description, success, checksum) \
         VALUES (1, 'init', 1, X'00');\
         INSERT INTO _sqlx_migrations (version, description, success, checksum) \
         VALUES (2, 'seed fiscal 2026', 1, X'00');",
    )
    .execute(&pool)
    .await
    .unwrap();
    sqlx::raw_sql(
        "DELETE FROM tablas_fiscales WHERE anio = 2026 AND tipo IN ('uma', 'subsidio_empleo');\
         INSERT INTO tablas_fiscales (anio, tipo, vigencia_inicio, vigencia_fin, datos_json) VALUES \
         (2026, 'uma', '2026-01-01', '2026-12-31', '{\"diaria\":113.14,\"mensual\":3439.46,\"anual\":41273.52}');\
         INSERT INTO tablas_fiscales (anio, tipo, vigencia_inicio, vigencia_fin, datos_json) VALUES \
         (2026, 'subsidio_empleo', '2026-01-01', '2026-12-31', '{\"maximo\":535.65,\"tope\":11492.66}');",
    )
    .execute(&pool)
    .await
    .unwrap();

    sqlx::query(
        "INSERT INTO empresas \
         (rfc, razon_social, regimen_fiscal, cp, estado_isn, tasa_isn, prima_riesgo_trabajo, registro_patronal_imss) \
         VALUES ('AAA010101AAA', 'Contador Prueba', '601', '00000', 'CIUDAD DE MEXICO', 3.0, 0.5, '')",
    )
    .execute(&pool)
    .await
    .unwrap();
    pool.close().await;

    run_migrations_async(&escenario.raiz).await.unwrap();

    let pool = conectar(&escenario.raiz).await;
    assert_eq!(versiones(&pool).await, vec![1, 2, 3, 4, 5, 6]);
    let legacy: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='_sqlx_migrations'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(legacy, 1, "el historial viejo se conserva como evidencia");

    let uma = filas_seed(&pool, "uma").await;
    assert_eq!(uma.len(), 2, "la fila vieja debe ser reemplazada");
    assert_eq!(uma[0].0, "2026-01-01");
    assert!(uma[0].1.contains("113.14"), "{}", uma[0].1);
    assert_eq!(uma[1].0, "2026-02-01");
    assert!(uma[1].1.contains("117.31"), "{}", uma[1].1);

    let subsidio = filas_seed(&pool, "subsidio_empleo").await;
    assert_eq!(subsidio.len(), 2, "la fila vieja debe ser reemplazada");
    assert!(subsidio[0].1.contains("536.21"), "{}", subsidio[0].1);
    assert!(subsidio[1].1.contains("535.65"), "{}", subsidio[1].1);

    let razon: String = sqlx::query_scalar(
        "SELECT razon_social FROM empresas WHERE rfc = 'AAA010101AAA'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(razon, "Contador Prueba");
}
