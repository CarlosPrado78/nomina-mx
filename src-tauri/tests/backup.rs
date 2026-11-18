// C1 — Evidencia de backup / restore de la base de datos SQLite.
//
// Referencia: art. 28 del CFF (conservación de documentación y registros) y
// guías del SAT sobre conservación de información contable. No hay fórmula
// fiscal que verificar: se comprueba que el respaldo sea íntegro, que el
// restore recupere los datos originales y que la operación sea reversible.
//
// sqlx 0.8 no expone `backup()`, por lo que el respaldo usa `VACUUM INTO`
// (forma SQL del online backup de SQLite). Cada test crea su propia BD.

use nomina_mx_lib::commands::backup::{
    crear_backup, crear_backup_con_stamp, listar_respaldos, restaurar_backup,
    validar_cabecera_sqlite,
};
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::SqlitePool;
use std::fs;
use std::path::{Path, PathBuf};

struct Escenario {
    raiz: PathBuf,
    db: PathBuf,
    backups: PathBuf,
    previos: PathBuf,
}

impl Escenario {
    fn nuevo(nombre: &str) -> Self {
        let nano = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let raiz = std::env::temp_dir().join(format!(
            "nomina_c1_{}_{}_{}",
            nombre,
            std::process::id(),
            nano
        ));
        if raiz.exists() {
            fs::remove_dir_all(&raiz).unwrap();
        }
        let db_dir = raiz.join("db");
        let backups = raiz.join("backups");
        let previos = raiz.join("previos");
        fs::create_dir_all(&db_dir).unwrap();
        fs::create_dir_all(&backups).unwrap();
        fs::create_dir_all(&previos).unwrap();
        Self {
            raiz,
            db: db_dir.join("nomina.db"),
            backups,
            previos,
        }
    }
}

impl Drop for Escenario {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.raiz);
    }
}

async fn pool(path: &Path, crear: bool) -> SqlitePool {
    SqlitePoolOptions::new()
        .connect_with(
            SqliteConnectOptions::new()
                .filename(path)
                .create_if_missing(crear),
        )
        .await
        .unwrap()
}

async fn crear_db_con_valor(path: &Path, valor: &str) {
    let p = pool(path, true).await;
    sqlx::query("CREATE TABLE IF NOT EXISTS empresa (id INTEGER PRIMARY KEY, nombre TEXT NOT NULL)")
        .execute(&p)
        .await
        .unwrap();
    sqlx::query("DELETE FROM empresa").execute(&p).await.unwrap();
    sqlx::query("INSERT INTO empresa (id, nombre) VALUES (1, ?)")
        .bind(valor)
        .execute(&p)
        .await
        .unwrap();
    p.close().await;
}

async fn valor_en(path: &Path) -> String {
    let p = pool(path, false).await;
    let valor: String = sqlx::query_scalar("SELECT nombre FROM empresa WHERE id = 1")
        .fetch_one(&p)
        .await
        .unwrap();
    p.close().await;
    valor
}

#[tokio::test]
async fn backup_genera_archivo_valido() {
    // Comportamiento esperado: `crear_backup` genera nomina_backup_YYYYMMDD_HHMMSS.db
    // con cabecera SQLite, `PRAGMA integrity_check = ok` y los mismos datos.
    // Referencia operativa: art. 28 CFF (conservación de información).
    let s = Escenario::nuevo("genera");
    crear_db_con_valor(&s.db, "Empresa Original").await;

    let ruta = crear_backup(&s.db, &s.backups).await.unwrap();
    assert!(ruta.exists(), "el archivo de respaldo debe existir");

    let nombre = ruta.file_name().unwrap().to_string_lossy();
    assert!(nombre.starts_with("nomina_backup_"));
    assert!(nombre.ends_with(".db"));
    let stamp = &nombre["nomina_backup_".len()..nombre.len() - ".db".len()];
    assert_eq!(stamp.len(), 15, "el stamp debe ser YYYYMMDD_HHMMSS: {}", stamp);
    assert!(stamp.chars().all(|c| c.is_ascii_digit() || c == '_'));

    validar_cabecera_sqlite(&ruta).unwrap();

    let p = pool(&ruta, false).await;
    let integridad: String = sqlx::query_scalar("PRAGMA integrity_check")
        .fetch_one(&p)
        .await
        .unwrap();
    assert_eq!(integridad.to_lowercase(), "ok");
    p.close().await;

    assert_eq!(valor_en(&ruta).await, "Empresa Original");

    const RESUMEN_BACKUP: &str = "listar_backups debe reportar nombre, fecha y tamaño";
    let lista = listar_respaldos(&s.backups).unwrap();
    assert_eq!(lista.len(), 1, "{}", RESUMEN_BACKUP);
    assert!(lista[0].tamano > 0, "{}", RESUMEN_BACKUP);
    assert_eq!(lista[0].fecha, stamp, "{}", RESUMEN_BACKUP);
}

#[tokio::test]
async fn restore_reemplaza_bd() {
    // Comportamiento esperado: backup → modificar datos → restore → datos
    // originales recuperados, y respaldo automático del estado modificado.
    // Referencia operativa: art. 28 CFF; el restore debe ser reversible.
    let s = Escenario::nuevo("reemplaza");
    crear_db_con_valor(&s.db, "Original").await;
    let backup = crear_backup(&s.db, &s.backups).await.unwrap();

    crear_db_con_valor(&s.db, "Modificado").await;
    assert_eq!(valor_en(&s.db).await, "Modificado");

    let respaldo_previo = restaurar_backup(&s.db, &backup, &s.previos).await.unwrap();

    assert_eq!(valor_en(&s.db).await, "Original");
    assert!(respaldo_previo.exists());
    assert_eq!(listar_respaldos(&s.previos).unwrap().len(), 1);
    assert_eq!(valor_en(&respaldo_previo).await, "Modificado");
}

#[tokio::test]
async fn restore_rechaza_archivo_corrupto() {
    // Comportamiento esperado: un archivo sin cabecera SQLite hace que
    // `restore_db` devuelva Err ANTES de tocar la BD activa ni crear un
    // respaldo previo. Referencia operativa: integridad del respaldo (art. 28 CFF).
    let s = Escenario::nuevo("corrupto");
    crear_db_con_valor(&s.db, "Original").await;

    let corrupto = s.backups.join("no_es_sqlite.db");
    fs::write(&corrupto, b"este archivo no tiene cabecera SQLite").unwrap();

    let err = restaurar_backup(&s.db, &corrupto, &s.previos)
        .await
        .unwrap_err();
    assert!(
        err.contains("cabecera SQLite"),
        "el error debe explicar la validación fallida: {}",
        err
    );
    assert_eq!(valor_en(&s.db).await, "Original");
    assert!(
        listar_respaldos(&s.previos).unwrap().is_empty(),
        "la validación falló antes del respaldo automático"
    );
}

#[tokio::test]
async fn backup_no_sobrescribe() {
    // Comportamiento esperado: dos backups con el MISMO timestamp generan
    // archivos distintos mediante sufijo (_1, _2, ...), sin pisar el primero.
    // Referencia operativa: conservación de todas las versiones (art. 28 CFF).
    let s = Escenario::nuevo("no_sobrescribe");
    crear_db_con_valor(&s.db, "Original").await;

    let primera = crear_backup_con_stamp(&s.db, &s.backups, "20260101_120000")
        .await
        .unwrap();
    let segunda = crear_backup_con_stamp(&s.db, &s.backups, "20260101_120000")
        .await
        .unwrap();

    assert_ne!(primera, segunda);
    assert!(primera.exists() && segunda.exists());
    assert_eq!(
        segunda.file_name().unwrap().to_string_lossy(),
        "nomina_backup_20260101_120000_1.db"
    );
    validar_cabecera_sqlite(&primera).unwrap();
    validar_cabecera_sqlite(&segunda).unwrap();

    let lista = listar_respaldos(&s.backups).unwrap();
    assert_eq!(lista.len(), 2);
    assert_eq!(valor_en(&primera).await, "Original");
    assert_eq!(valor_en(&segunda).await, "Original");
}
