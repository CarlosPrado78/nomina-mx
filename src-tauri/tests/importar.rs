// C2 — Evidencia de importación masiva de trabajadores desde CSV.
//
// Fuentes: CFF art. 28 (conservación de registros), LFT arts. 2 y 25
// (relación de trabajo y salario) y catálogos SAT de PeriodicidadPago y
// TipoContrato usados por el CFDI de Nómina 1.2.
//
// Cálculo manual verificado en cada prueba:
//   - 5 filas válidas => 5 INSERT (uno por fila) y 5 registros en BD.
//   - 5 filas con 1 CURP inválida => 4 INSERT y 1 error en fila 3.
//   - Falta columna requerida => Err de archivo, 0 INSERT.
//   - RFC inválido => 0 INSERT para esa fila, error en columna `rfc`.
//   - Latin-1 => mismo contenido acentuado que UTF-8.
//
// Las migraciones reales 0001–0004 se aplican sobre SQLite temporal.

use encoding_rs::WINDOWS_1252;
use nomina_mx_lib::commands::importar::importar_trabajadores_csv_pool;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::SqlitePool;
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

async fn escenario(nombre: &str) -> Escenario {
    let nano = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let raiz = std::env::temp_dir().join(format!(
        "nomina_c2_{}_{}_{}",
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
    for sql in [
        include_str!("../migrations/0001_init.sql"),
        include_str!("../migrations/0002_seed_fiscal_2026.sql"),
        include_str!("../migrations/0003_horas_extra.sql"),
        include_str!("../migrations/0003_audit_log.sql"),
        include_str!("../migrations/0004_multi_empresa.sql"),
    ] {
        sqlx::raw_sql(sql).execute(&pool).await.unwrap();
    }
    Escenario { raiz, pool }
}

impl Escenario {
    /// Escribe un CSV UTF-8 (con BOM opcional) y devuelve su ruta.
    fn csv(&self, nombre: &str, contenido: &str) -> String {
        let ruta = self.raiz.join(nombre);
        fs::write(&ruta, contenido.as_bytes()).unwrap();
        ruta.to_string_lossy().into_owned()
    }

    /// Escribe un CSV en Windows-1252 (Latin-1 extendido usado por Excel).
    fn csv_latin1(&self, nombre: &str, contenido: &str) -> String {
        let (bytes, _, _) = WINDOWS_1252.encode(contenido);
        let ruta = self.raiz.join(nombre);
        fs::write(&ruta, bytes.as_ref()).unwrap();
        ruta.to_string_lossy().into_owned()
    }

    async fn total_trabajadores(&self) -> i64 {
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM trabajadores")
            .fetch_one(&self.pool)
            .await
            .unwrap()
    }
}

const ENCABEZADO: &str = "nombre,curp,rfc,nss,fecha_ingreso,salario_diario,periodicidad_pago,tipo_contrato";

/// Cinco filas válidas con datos fiscales distintos. Se agrega la columna
/// `empresa_id` con un valor falso (999) para comprobar que se ignora.
fn filas_validas() -> Vec<String> {
    vec![
        "Ana RODRIGUEZ,RODR900101HDFNSR08,RODR900101AB1,12345678901,2020-01-15,450.50,mensual,indefinido".into(),
        "Berta GARCIA,GARC850512MDFNSR09,GARC850512CD2,12345678902,2019-03-20,500.00,quincenal,02".into(),
        "Carlos LOPEZ,LOPE770423HDFNSR10,LOPE770423EF3,12345678903,2021-07-05,600.75,semanal,temporal".into(),
        "Dora MARTINEZ,MART920714HDFNSR11,MART920714GH4,12345678904,2022-11-30,750.00,mensual,obra_determinada".into(),
        "Elena HERNANDEZ,HERN881129MDFNSR12,HERN881129IJ5,12345678905,2018-09-01,900.25,mensual,01".into(),
    ]
}

fn csv_con(encabezado: &str, filas: &[String]) -> String {
    let mut lineas = vec![encabezado.to_string()];
    lineas.extend(filas.iter().cloned());
    format!("{}\n", lineas.join("\n"))
}

// ---------------------------------------------------------------------------
// 1. importar_csv_valido_inserta_todos (5 filas → 5 insertados)
// ---------------------------------------------------------------------------
#[tokio::test]
async fn importar_csv_valido_inserta_todos() {
    let esc = escenario("valido").await;
    let mut filas = filas_validas();
    // `empresa_id` NO debe tomarse del CSV: se fuerza un valor incorrecto.
    let encabezado = format!("{},empresa_id", ENCABEZADO);
    for f in filas.iter_mut() {
        f.push_str(",999");
    }
    let ruta = esc.csv("valido.csv", &csv_con(&encabezado, &filas));

    let res = importar_trabajadores_csv_pool(&esc.pool, &ruta)
        .await
        .expect("CSV válido no debe fallar a nivel de archivo");

    assert_eq!(res.insertados, 5, "5 filas válidas => 5 insertados");
    assert!(res.errores.is_empty(), "no deben haber errores de fila");
    assert_eq!(esc.total_trabajadores().await, 5, "5 registros en BD");

    let empresa_id: i64 = sqlx::query_scalar("SELECT empresa_id FROM trabajadores LIMIT 1")
        .fetch_one(&esc.pool)
        .await
        .unwrap();
    assert_eq!(empresa_id, 1, "empresa_id se asigna desde la empresa activa (1), no del CSV (999)");

    let audit: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM audit_log WHERE tabla = 'trabajadores' AND operacion = 'INSERT'",
    )
    .fetch_one(&esc.pool)
    .await
    .unwrap();
    assert_eq!(audit, 5, "cada INSERT queda registrado en audit_log");
}

// ---------------------------------------------------------------------------
// 2. importar_csv_con_una_fila_invalida_procesa_el_resto
// ---------------------------------------------------------------------------
#[tokio::test]
async fn importar_csv_con_una_fila_invalida_procesa_el_resto() {
    let esc = escenario("fila_invalida").await;
    let mut filas = filas_validas();
    // Fila de datos 2 => fila CSV 3 (encabezado = 1).
    filas[1] = "Berta GARCIA,CURP_MALA,GARC850512CD2,12345678902,2019-03-20,500.00,quincenal,02".into();
    let ruta = esc.csv("fila_invalida.csv", &csv_con(ENCABEZADO, &filas));

    let res = importar_trabajadores_csv_pool(&esc.pool, &ruta)
        .await
        .expect("una fila mala no debe abortar el archivo");

    assert_eq!(res.insertados, 4, "las otras 4 filas se insertan");
    assert_eq!(res.errores.len(), 1, "se reporta exactamente 1 error");
    assert_eq!(res.errores[0].fila, 3, "el error indica la fila CSV 3");
    assert_eq!(res.errores[0].columna, "curp", "el error indica la columna curp");
    assert_eq!(esc.total_trabajadores().await, 4, "4 registros en BD");
}

// ---------------------------------------------------------------------------
// 3. importar_csv_falta_columna_requerida_falla
// ---------------------------------------------------------------------------
#[tokio::test]
async fn importar_csv_falta_columna_requerida_falla() {
    let esc = escenario("falta_columna").await;
    let encabezado = ENCABEZADO.replace("curp,", "");
    let filas = filas_validas();
    let ruta = esc.csv("falta_curp.csv", &csv_con(&encabezado, &filas));

    let res = importar_trabajadores_csv_pool(&esc.pool, &ruta).await;

    assert!(res.is_err(), "falta una columna requerida => Err de archivo");
    let msg = res.unwrap_err();
    assert!(msg.contains("falta la columna requerida: curp"), "mensaje: {}", msg);
    assert_eq!(esc.total_trabajadores().await, 0, "ninguna fila se inserta");
}

// ---------------------------------------------------------------------------
// 4. importar_csv_rfc_invalido_no_inserta
// ---------------------------------------------------------------------------
#[tokio::test]
async fn importar_csv_rfc_invalido_no_inserta() {
    let esc = escenario("rfc_invalido").await;
    let fila = "Ana RODRIGUEZ,RODR900101HDFNSR08,RFC_MALO,12345678901,2020-01-15,450.50,mensual,indefinido";
    let ruta = esc.csv("rfc_invalido.csv", &csv_con(ENCABEZADO, &[fila.to_string()]));

    let res = importar_trabajadores_csv_pool(&esc.pool, &ruta)
        .await
        .expect("error de fila no es error de archivo");

    assert_eq!(res.insertados, 0, "RFC inválido => fila rechazada");
    assert_eq!(res.errores.len(), 1);
    assert_eq!(res.errores[0].fila, 2, "primera fila de datos = fila CSV 2");
    assert_eq!(res.errores[0].columna, "rfc");
    assert_eq!(esc.total_trabajadores().await, 0, "0 registros en BD");
}

// ---------------------------------------------------------------------------
// 5. importar_csv_encoding_latin1_funciona
// ---------------------------------------------------------------------------
#[tokio::test]
async fn importar_csv_encoding_latin1_funciona() {
    let esc = escenario("latin1").await;
    let fila = "José Ángel,RODR900101HDFNSR08,RODR900101AB1,12345678901,2020-01-15,450.50,mensual,indefinido";
    let ruta = esc.csv_latin1("latin1.csv", &csv_con(ENCABEZADO, &[fila.to_string()]));

    let res = importar_trabajadores_csv_pool(&esc.pool, &ruta)
        .await
        .expect("Latin-1 debe decodificarse con fallback");

    assert_eq!(res.insertados, 1, "la fila Latin-1 se inserta");
    assert!(res.errores.is_empty());
    let nombre: String = sqlx::query_scalar("SELECT nombre FROM trabajadores LIMIT 1")
        .fetch_one(&esc.pool)
        .await
        .unwrap();
    assert_eq!(nombre, "José Ángel", "los acentos Latin-1 se conservan");
}
