// C5 — Evidencia de recálculo retroactivo de SBC y recibos.
//
// Fuentes y cálculo manual:
//   - CFF art. 28: conservación; cada recálculo deja fila en audit_log.
//   - LSS arts. 27-28: SDI = salario × 1.04965…; tope = 25 × UMA.
//     Enero 2026: 25 × 113.14 = 2,828.50 exacto.
//   - LISR art. 96 y RMF Anexo 8: con ingreso 10,500 el ISR bruto mensual cae
//     en el tramo 7,168.52–12,598.02: 420.95 + (10,500 − 7,168.52) × 10.88%
//     = 783.415024. El subsidio de enero (536.21) deja ISR 247.205024; el de
//     febrero (535.65) deja 247.765024. Diferencia exacta: 0.56 enero vs feb.
//   - CFDI 4.0: un recibo con uuid_cfdi es inmutable.
//
// Se aplican las migraciones reales 1–5 sobre SQLite temporal.

use nomina_mx_lib::commands::nomina::{guardar_periodo_pool, guardar_recibo_pool, PeriodoNomina, ReciboCalculado};
use nomina_mx_lib::commands::recalculo::{
    calcular_recibo_historico_pool, recalcular_periodo_pool, recalcular_trabajador_periodo_pool,
};
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
        "nomina_c5_{}_{}_{}",
        nombre,
        std::process::id(),
        nano
    ));
    if raiz.exists() {
        fs::remove_dir_all(&raiz).unwrap();
    }
    fs::create_dir_all(&raiz).unwrap();
    let db = raiz.join("nomina.db");
    let pool = SqlitePoolOptions::new()
        .connect_with(
            SqliteConnectOptions::new()
                .filename(&db)
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
    async fn trabajador(&self, nombre: &str, salario: f64) -> i64 {
        let res = sqlx::query(
            "INSERT INTO trabajadores (
               empresa_id, nombre, curp, rfc, nss, fecha_ingreso, tipo_contrato,
               salario_diario, periodicidad_pago, jornada, horas_semanales,
               infonavit_activo, infonavit_factor, infonavit_tipo_descuento
             ) VALUES (1, ?, '', '', '', '2020-01-01', 'indefinido', ?,
                       'mensual', 'diurna', 40, 0, 0, 'CUOTA_FIJA_VSM')",
        )
        .bind(nombre)
        .bind(salario)
        .execute(&self.pool)
        .await
        .unwrap();
        res.last_insert_rowid()
    }

    async fn periodo(&self, fecha_pago: &str, dias: i64, estado: &str) -> i64 {
        let periodo = PeriodoNomina {
            id: None,
            empresa_id: 1,
            tipo: "mensual".into(),
            fecha_inicio: "2026-01-01".into(),
            fecha_fin: "2026-01-31".into(),
            fecha_pago: fecha_pago.into(),
            estado: estado.into(),
            dias,
        };
        guardar_periodo_pool(&self.pool, &periodo).await.unwrap()
    }

    async fn recibo(&self, periodo_id: i64, r: &ReciboCalculado) -> i64 {
        guardar_recibo_pool(&self.pool, periodo_id, r).await.unwrap()
    }

    async fn calcular(&self, trabajador_id: i64, dias: i64, fecha_pago: &str) -> ReciboCalculado {
        calcular_recibo_historico_pool(&self.pool, 1, trabajador_id, dias, fecha_pago, None)
            .await
            .unwrap()
    }

    async fn audit_json(&self, periodo_id: i64) -> serde_json::Value {
        let fila: (String,) = sqlx::query_as(
            "SELECT despues_json FROM audit_log \
             WHERE operacion='RECALCULO' AND registro_id=? ORDER BY id DESC LIMIT 1",
        )
        .bind(periodo_id)
        .fetch_one(&self.pool)
        .await
        .unwrap();
        serde_json::from_str(&fila.0).unwrap()
    }
}

fn cerca(a: f64, b: f64, tol: f64) -> bool {
    (a - b).abs() < tol
}

// ---------------------------------------------------------------------------
// 1. recalculo_usa_uma_historica
// ---------------------------------------------------------------------------
#[tokio::test]
async fn recalculo_usa_uma_historica() {
    let esc = escenario("uma_historica").await;
    let trabajador = esc.trabajador("Ana Tobo", 10_000.0).await;
    let periodo = esc.periodo("2026-01-31", 30, "cerrado").await;

    // Original calculado CON UMA ACTUAL (noviembre): tope 25 × 117.31.
    let actual = esc.calcular(trabajador, 30, "2026-11-15").await;
    assert!(cerca(actual.sbc_diario, 2_932.75, 0.001));
    esc.recibo(periodo, &actual).await;

    // Cifra manual de enero: tope LSS art. 28 = 25 × 113.14 = 2,828.50.
    let historico = esc.calcular(trabajador, 30, "2026-01-31").await;
    assert!(cerca(historico.sbc_diario, 2_828.50, 0.001));

    let res = recalcular_periodo_pool(&esc.pool, periodo, "Corrección UMA enero")
        .await
        .unwrap();

    assert_eq!(res.recibos_recalculados, 1);
    assert!(!res.diferencias.is_empty(), "el cambio de UMA debe generar diferencia");
    let guardado: (f64,) = sqlx::query_as("SELECT neto FROM recibos WHERE periodo_id=?")
        .bind(periodo)
        .fetch_one(&esc.pool)
        .await
        .unwrap();
    assert!(cerca(guardado.0, historico.neto, 0.01), "el recibo debe quedar con cifras de enero");

    let auditoria = esc.audit_json(periodo).await;
    let uma = auditoria["parametros_vigentes"]["uma_diaria"].as_f64().unwrap();
    assert!(cerca(uma, 113.14, 0.0001), "la auditoría debe registrar UMA 113.14, no 117.31");
    assert_eq!(
        auditoria["fecha_pago_original"].as_str().unwrap(),
        "2026-01-31"
    );
}

// ---------------------------------------------------------------------------
// 2. recalculo_con_subsidio_enero_usa_536_21
// ---------------------------------------------------------------------------
#[tokio::test]
async fn recalculo_con_subsidio_enero_usa_536_21() {
    let esc = escenario("subsidio_enero").await;
    let trabajador = esc.trabajador("BertaSubsidio", 350.0).await;
    let periodo = esc.periodo("2026-01-31", 30, "cerrado").await;

    let historico = esc.calcular(trabajador, 30, "2026-01-31").await;
    // ISR manual enero: 783.415024 − 536.21 = 247.205024.
    assert!(cerca(historico.isr, 247.205024, 0.0001), "ISR enero {}", historico.isr);

    let febrero = esc.calcular(trabajador, 30, "2026-02-15").await;
    // ISR manual febrero: 783.415024 − 535.65 = 247.765024.
    assert!(cerca(febrero.isr, 247.765024, 0.0001), "ISR febrero {}", febrero.isr);

    // Recibo original con el subsidio de febrero (0.56 de ISR de más).
    let mut original = historico.clone();
    original.isr = febrero.isr;
    original.total_deducciones = historico.total_deducciones + 0.56;
    original.neto = historico.neto - 0.56;
    esc.recibo(periodo, &original).await;

    let res = recalcular_periodo_pool(&esc.pool, periodo, "Subsidio enero")
        .await
        .unwrap();

    assert_eq!(res.recibos_recalculados, 1);
    let d = &res.diferencias[0];
    assert!(cerca(d.delta_neto, 0.56, 0.01), "delta esperado 0.56, obtenido {}", d.delta_neto);
    assert!(cerca(d.neto_despues, historico.neto, 0.01));

    let auditoria = esc.audit_json(periodo).await;
    let subsidio = auditoria["parametros_vigentes"]["subsidio_maximo"]
        .as_f64()
        .unwrap();
    assert!(cerca(subsidio, 536.21, 0.0001), "debe usar 536.21, no 535.65");
}

// ---------------------------------------------------------------------------
// 3. recalculo_reporta_diferencias
// ---------------------------------------------------------------------------
#[tokio::test]
async fn recalculo_reporta_diferencias() {
    let esc = escenario("diferencias").await;
    let modificado = esc.trabajador("CarlosDiff", 350.0).await;
    let intacto = esc.trabajador("DoraIgual", 500.0).await;
    let periodo = esc.periodo("2026-01-31", 30, "cerrado").await;

    let r1 = esc.calcular(modificado, 30, "2026-01-31").await;
    let r2 = esc.calcular(intacto, 30, "2026-01-31").await;
    esc.recibo(periodo, &r1).await;
    esc.recibo(periodo, &r2).await;

    // Cambio de salario posterior al cierre del periodo.
    sqlx::query("UPDATE trabajadores SET salario_diario=? WHERE id=?")
        .bind(400.0)
        .bind(modificado)
        .execute(&esc.pool)
        .await
        .unwrap();

    let res = recalcular_periodo_pool(&esc.pool, periodo, "Salario corregido")
        .await
        .unwrap();

    assert_eq!(res.recibos_recalculados, 1);
    assert_eq!(res.diferencias.len(), 1, "solo el trabajador modificado debe cambiar");
    let d = &res.diferencias[0];
    assert_eq!(d.trabajador_id, modificado);
    assert_eq!(d.nombre, "CarlosDiff");
    assert!(cerca(d.percepciones_despues, 400.0 * 30.0, 0.01));
    assert!(cerca(d.delta_neto, d.neto_despues - d.neto_antes, 0.0001));

    let esperado = esc.calcular(modificado, 30, "2026-01-31").await;
    assert!(cerca(d.neto_despues, esperado.neto, 0.01));

    // La ruta individual también funciona y respeta el motivo de sistema.
    let individual = recalcular_trabajador_periodo_pool(&esc.pool, intacto, periodo)
        .await
        .unwrap();
    assert_eq!(individual.recibos_recalculados, 0, "sin cambios no hay UPDATE");
}

// ---------------------------------------------------------------------------
// 4. recalculo_registra_auditoria
// ---------------------------------------------------------------------------
#[tokio::test]
async fn recalculo_registra_auditoria() {
    let esc = escenario("auditoria").await;
    let trabajador = esc.trabajador("ElenaAudit", 350.0).await;
    let periodo = esc.periodo("2026-01-31", 30, "cerrado").await;
    let original = esc.calcular(trabajador, 30, "2026-01-31").await;
    esc.recibo(periodo, &original).await;

    // Motivo vacío o periodo inexistente => Err.
    assert!(recalcular_periodo_pool(&esc.pool, periodo, "   ").await.is_err());
    assert!(recalcular_periodo_pool(&esc.pool, 999_999, "Motivo válido").await.is_err());

    let motivo = "Corrección de catálogo 2026-11";
    let res = recalcular_periodo_pool(&esc.pool, periodo, motivo).await.unwrap();
    assert_eq!(res.recibos_recalculados, 0, "sin cambios no hay UPDATE");

    let total: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM audit_log WHERE operacion='RECALCULO' AND tabla='recibos'",
    )
    .fetch_one(&esc.pool)
    .await
    .unwrap();
    assert_eq!(total.0, 1, "debe existir exactamente una fila RECALCULO");

    let auditoria = esc.audit_json(periodo).await;
    assert_eq!(auditoria["motivo"].as_str().unwrap(), motivo);
    assert_eq!(auditoria["cfdi_ajuste"].as_str().unwrap(), "PENDIENTE_V1_1");
}

// ---------------------------------------------------------------------------
// 5. recalculo_no_modifica_cfdi_timbrado
// ---------------------------------------------------------------------------
#[tokio::test]
async fn recalculo_no_modifica_cfdi_timbrado() {
    let esc = escenario("cfdi_timbrado").await;
    let trabajador = esc.trabajador("FaustinoTimbre", 350.0).await;
    let periodo = esc.periodo("2026-01-31", 30, "cerrado").await;
    let original = esc.calcular(trabajador, 30, "2026-01-31").await;
    let recibo_id = esc.recibo(periodo, &original).await;

    sqlx::query("UPDATE recibos SET uuid_cfdi=?, xml_cfdi=?, fecha_timbrado=? WHERE id=?")
        .bind("ABC-1234")
        .bind("<cfdi original/>")
        .bind("2026-02-01T10:00:00Z")
        .bind(recibo_id)
        .execute(&esc.pool)
        .await
        .unwrap();

    // Fuerza un cambio para comprobar que SE REPORTa pero NO se aplica.
    sqlx::query("UPDATE trabajadores SET salario_diario=? WHERE id=?")
        .bind(500.0)
        .bind(trabajador)
        .execute(&esc.pool)
        .await
        .unwrap();

    let res = recalcular_periodo_pool(&esc.pool, periodo, "Ajuste con CFDI timbrado")
        .await
        .unwrap();

    assert_eq!(res.recibos_recalculados, 0);
    assert_eq!(res.diferencias.len(), 1, "la diferencia se reporta igual");

    let fila: (Option<String>, Option<String>, f64) = sqlx::query_as(
        "SELECT uuid_cfdi, xml_cfdi, total_percepciones FROM recibos WHERE id=?",
    )
    .bind(recibo_id)
    .fetch_one(&esc.pool)
    .await
    .unwrap();
    assert_eq!(fila.0.as_deref(), Some("ABC-1234"));
    assert_eq!(fila.1.as_deref(), Some("<cfdi original/>"));
    assert!(cerca(fila.2, original.total_percepciones, 0.001), "el importe no debe cambiar");
}
