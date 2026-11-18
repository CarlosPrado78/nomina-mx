// C4 — Evidencia de multi-empresa con selección persistida.
//
// Fuentes: LISR art. 79 fracc. III (requisitos de deducción de nómina) y
// CFF art. 27 (contabilidad por establecimiento). Se aplican las migraciones
// reales (1..5) sobre SQLite temporal, igual que en `tests/auditoria.rs`.

use nomina_mx_lib::commands::empresas::{
    cambiar_empresa_activa_pool, crear_empresa_pool, empresa_activa_pool,
    listar_empresas_pool, NuevaEmpresa,
};
use nomina_mx_lib::commands::nomina::{
    guardar_periodo_pool, guardar_recibo_pool, listar_recibos_pool, PeriodoNomina,
    ReciboCalculado,
};
use nomina_mx_lib::commands::trabajador::{
    guardar_trabajador_pool, listar_trabajadores_pool, Trabajador,
};
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::SqlitePool;
use std::fs;
use std::path::PathBuf;

struct Escenario {
    raiz: PathBuf,
    ruta_db: PathBuf,
    pool: SqlitePool,
}

impl Drop for Escenario {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.raiz);
    }
}

fn ruta_db(nombre: &str) -> (PathBuf, PathBuf) {
    let nano = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let raiz = std::env::temp_dir().join(format!(
        "nomina_c4_{}_{}_{}",
        nombre,
        std::process::id(),
        nano
    ));
    if raiz.exists() {
        fs::remove_dir_all(&raiz).unwrap();
    }
    fs::create_dir_all(&raiz).unwrap();
    let db = raiz.join("nomina.db");
    (raiz, db)
}

async fn conectar(ruta: &PathBuf) -> SqlitePool {
    SqlitePoolOptions::new()
        .connect_with(
            SqliteConnectOptions::new()
                .filename(ruta)
                .create_if_missing(true),
        )
        .await
        .unwrap()
}

/// Migraciones 1–4 (estado anterior a C4: sin config, sin empresa_id en
/// recibos/incapacidades/vacaciones).
async fn aplicar_base(pool: &SqlitePool) {
    for sql in [
        include_str!("../migrations/0001_init.sql"),
        include_str!("../migrations/0002_seed_fiscal_2026.sql"),
        include_str!("../migrations/0003_horas_extra.sql"),
        include_str!("../migrations/0003_audit_log.sql"),
    ] {
        sqlx::raw_sql(sql).execute(pool).await.unwrap();
    }
}

async fn aplicar_0004(pool: &SqlitePool) {
    sqlx::raw_sql(include_str!("../migrations/0004_multi_empresa.sql"))
        .execute(pool)
        .await
        .unwrap();
}

async fn db_nuevo(nombre: &str) -> Escenario {
    let (raiz, db) = ruta_db(nombre);
    let pool = conectar(&db).await;
    aplicar_base(&pool).await;
    aplicar_0004(&pool).await;
    Escenario { raiz, ruta_db: db, pool }
}

fn nueva_empresa(rfc: &str, razon: &str) -> NuevaEmpresa {
    NuevaEmpresa {
        rfc: rfc.to_string(),
        razon_social: razon.to_string(),
        regimen_fiscal: "601".into(),
        cp: "00000".into(),
        estado_isn: "CIUDAD DE MEXICO".into(),
        tasa_isn: 3.0,
        prima_riesgo_trabajo: 0.5,
        registro_patronal_imss: String::new(),
    }
}

fn trabajador_de_prueba(empresa_id: i64, nombre: &str) -> Trabajador {
    Trabajador {
        id: None,
        empresa_id, // valor "del formulario" que debe ser ignorado
        nombre: nombre.to_string(),
        curp: String::new(),
        rfc: String::new(),
        nss: String::new(),
        num_empleado: String::new(),
        fecha_nacimiento: "1990-01-01".into(),
        estado_civil: "SOLTERO".into(),
        fecha_ingreso: "2020-01-01".into(),
        tipo_contrato: "INDEFINIDO".into(),
        departamento: String::new(),
        puesto: String::new(),
        salario_diario: 400.0,
        periodicidad_pago: "mensual".into(),
        jornada: "diurna".into(),
        horas_semanales: 40.0,
        infonavit_activo: false,
        infonavit_num_credito: None,
        infonavit_factor: 0.0,
        infonavit_tipo_descuento: "CUOTA_FIJA_VSM".into(),
        fecha_baja: None,
        tipo_baja: None,
    }
}

fn periodo_de_prueba(empresa_id: i64) -> PeriodoNomina {
    PeriodoNomina {
        id: None,
        empresa_id, // valor "del formulario" que debe ser ignorado
        tipo: "mensual".into(),
        fecha_inicio: "2026-01-01".into(),
        fecha_fin: "2026-01-31".into(),
        fecha_pago: "2026-01-31".into(),
        estado: "cerrado".into(),
        dias: 31,
    }
}

async fn insertar_trabajador(pool: &SqlitePool, empresa_id: i64, nombre: &str) -> i64 {
    sqlx::query("INSERT INTO trabajadores (empresa_id, nombre, salario_diario) VALUES (?, ?, 400)")
        .bind(empresa_id)
        .bind(nombre)
        .execute(pool)
        .await
        .unwrap()
        .last_insert_rowid()
}

#[tokio::test]
async fn multi_empresa_aislamiento_trabajadores() {
    // Esperado: con empresa 1 activa solo se listan sus trabajadores; al
    // cambiar a la empresa 2 solo se listan los de esa empresa. Además,
    // guardar_trabajador ignora el empresa_id recibido del formulario.
    // Regla: C4 / LISR art. 79 III.
    let s = db_nuevo("aislamiento_trabajadores").await;
    let empresa2 = crear_empresa_pool(&s.pool, &nueva_empresa("BBBB010101ABC", "Empresa Dos"))
        .await
        .unwrap();
    let id2 = empresa2.id.unwrap();

    insertar_trabajador(&s.pool, 1, "Trabajador Uno").await;
    insertar_trabajador(&s.pool, id2, "Trabajador Dos").await;

    cambiar_empresa_activa_pool(&s.pool, 1).await.unwrap();
    let lista = listar_trabajadores_pool(&s.pool).await.unwrap();
    assert_eq!(lista.len(), 1, "empresa 1 debe ver solo su trabajador");
    assert_eq!(lista[0].nombre, "Trabajador Uno");
    assert!(lista.iter().all(|t| t.empresa_id == 1));

    cambiar_empresa_activa_pool(&s.pool, id2).await.unwrap();
    let lista = listar_trabajadores_pool(&s.pool).await.unwrap();
    assert_eq!(lista.len(), 1, "empresa 2 debe ver solo su trabajador");
    assert_eq!(lista[0].nombre, "Trabajador Dos");
    assert!(lista.iter().all(|t| t.empresa_id == id2));

    // El formulario envía empresa_id=1 mientras la activa es la empresa 2.
    let id = guardar_trabajador_pool(&s.pool, &trabajador_de_prueba(1, "Forzado"))
        .await
        .unwrap();
    let guardado: i64 = sqlx::query_scalar("SELECT empresa_id FROM trabajadores WHERE id = ?")
        .bind(id)
        .fetch_one(&s.pool)
        .await
        .unwrap();
    assert_eq!(guardado, id2, "guardar_trabajador debe ignorar el empresa_id del formulario");
}

#[tokio::test]
async fn multi_empresa_aislamiento_recibos() {
    // Esperado: listar_recibos_pool devuelve solo los recibos de la empresa
    // activa. guardar_recibo_pool sella empresa_id con la activa y
    // guardar_periodo_pool ignora el empresa_id del formulario.
    // Regla: C4 / CFF art. 27.
    let s = db_nuevo("aislamiento_recibos").await;
    let empresa2 = crear_empresa_pool(&s.pool, &nueva_empresa("BBBB010101ABC", "Empresa Dos"))
        .await
        .unwrap();
    let id2 = empresa2.id.unwrap();

    cambiar_empresa_activa_pool(&s.pool, 1).await.unwrap();
    let t1 = insertar_trabajador(&s.pool, 1, "Uno").await;
    let p1 = guardar_periodo_pool(&s.pool, &periodo_de_prueba(999))
        .await
        .unwrap();
    guardar_recibo_pool(
        &s.pool,
        p1,
        &ReciboCalculado { trabajador_id: t1, neto: 1000.0, ..Default::default() },
    )
    .await
    .unwrap();

    cambiar_empresa_activa_pool(&s.pool, id2).await.unwrap();
    let t2 = insertar_trabajador(&s.pool, id2, "Dos").await;
    let p2 = guardar_periodo_pool(&s.pool, &periodo_de_prueba(1))
        .await
        .unwrap();
    guardar_recibo_pool(
        &s.pool,
        p2,
        &ReciboCalculado { trabajador_id: t2, neto: 2000.0, ..Default::default() },
    )
    .await
    .unwrap();

    let empresa_p1: i64 = sqlx::query_scalar("SELECT empresa_id FROM periodos_nomina WHERE id = ?")
        .bind(p1).fetch_one(&s.pool).await.unwrap();
    assert_eq!(empresa_p1, 1, "guardar_periodo_pool ignora el empresa_id del formulario");

    cambiar_empresa_activa_pool(&s.pool, 1).await.unwrap();
    let recibos = listar_recibos_pool(&s.pool).await.unwrap();
    assert_eq!(recibos.len(), 1);
    assert_eq!(recibos[0].empresa_id, 1);
    assert_eq!(recibos[0].neto, 1000.0);

    cambiar_empresa_activa_pool(&s.pool, id2).await.unwrap();
    let recibos = listar_recibos_pool(&s.pool).await.unwrap();
    assert_eq!(recibos.len(), 1);
    assert_eq!(recibos[0].empresa_id, id2);
    assert_eq!(recibos[0].neto, 2000.0);
}

#[tokio::test]
async fn cambiar_empresa_activa_persiste() {
    // Esperado: el ID queda en config(clave='empresa_activa') y sobrevive a
    // una nueva conexión. Cambiar a un ID inexistente debe fallar.
    let s = db_nuevo("persiste").await;
    let empresa2 = crear_empresa_pool(&s.pool, &nueva_empresa("BBBB010101ABC", "Empresa Dos"))
        .await
        .unwrap();
    let id2 = empresa2.id.unwrap();

    cambiar_empresa_activa_pool(&s.pool, id2).await.unwrap();
    assert_eq!(empresa_activa_pool(&s.pool).await.unwrap(), id2);

    let empresas = listar_empresas_pool(&s.pool).await.unwrap();
    assert_eq!(empresas.iter().filter(|e| e.activa).count(), 1);
    assert_eq!(empresas.iter().find(|e| e.activa).unwrap().id, Some(id2));

    // Nueva conexión al MISMO archivo: comprueba persistencia real.
    let pool2 = SqlitePoolOptions::new()
        .connect_with(
            SqliteConnectOptions::new()
                .filename(&s.ruta_db)
                .create_if_missing(false),
        )
        .await
        .unwrap();
    let valor: String = sqlx::query_scalar("SELECT valor FROM config WHERE clave = 'empresa_activa'")
        .fetch_one(&pool2)
        .await
        .unwrap();
    assert_eq!(valor, id2.to_string());
    assert_eq!(empresa_activa_pool(&pool2).await.unwrap(), id2);

    assert!(
        cambiar_empresa_activa_pool(&s.pool, 9999).await.is_err(),
        "no se puede activar una empresa inexistente"
    );
    assert_eq!(empresa_activa_pool(&s.pool).await.unwrap(), id2);
}

#[tokio::test]
async fn crear_empresa_valida_rfc_unico() {
    // Esperado: crear_empresa_pool rechaza un RFC ya existente y la BD
    // (índice ux_empresas_rfc) rechaza incluso un INSERT directo duplicado.
    let s = db_nuevo("rfc_unico").await;

    let err = crear_empresa_pool(&s.pool, &nueva_empresa("XAXX010101000", "Duplicada Default"))
        .await
        .unwrap_err();
    assert!(err.to_lowercase().contains("rfc"), "{}", err);

    let creada = crear_empresa_pool(&s.pool, &nueva_empresa("BBBB010101ABC", "Empresa Nueva"))
        .await
        .unwrap();
    assert!(creada.id.is_some());
    assert_eq!(creada.razon_social, "Empresa Nueva");
    assert!(!creada.activa);

    let duplicado = sqlx::query("INSERT INTO empresas (rfc, razon_social) VALUES (?, ?)")
        .bind("BBBB010101ABC")
        .bind("Otra con el mismo RFC")
        .execute(&s.pool)
        .await;
    assert!(duplicado.is_err(), "el índice UNIQUE de la BD debe rechazar el RFC duplicado");

    let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM empresas WHERE rfc = 'BBBB010101ABC'")
        .fetch_one(&s.pool)
        .await
        .unwrap();
    assert_eq!(total, 1);
}

#[tokio::test]
async fn migracion_0004_asigna_default_a_registros_existentes() {
    // Esperado: todos los registros creados ANTES de la migración quedan con
    // empresa_id=1, se crea la Empresa Default si no existía y la selección
    // queda persistida en config.
    // Regla: C4 (asignación masiva a Empresa Default).
    let (raiz, db) = ruta_db("migracion_default");
    let pool = conectar(&db).await;
    aplicar_base(&pool).await;

    // Datos "legados": empresa 1, trabajador, periodo, recibo, incapacidad y
    // vacaciones. Las tres últimas tablas aún no tenían empresa_id.
    sqlx::query("INSERT INTO empresas (id, rfc, razon_social) VALUES (1, 'LEGACY010101AA1', 'Empresa Legacy')")
        .execute(&pool).await.unwrap();
    let trabajador = sqlx::query("INSERT INTO trabajadores (empresa_id, nombre, salario_diario) VALUES (1, 'Legacy', 400)")
        .execute(&pool).await.unwrap().last_insert_rowid();
    let periodo: i64 = sqlx::query("INSERT INTO periodos_nomina (empresa_id, tipo, fecha_inicio, fecha_fin, fecha_pago, estado, dias) VALUES (1, 'mensual', '2026-01-01', '2026-01-31', '2026-01-31', 'cerrado', 31)")
        .execute(&pool).await.unwrap().last_insert_rowid();
    let recibo: i64 = sqlx::query("INSERT INTO recibos (periodo_id, trabajador_id, neto) VALUES (?, ?, 1000)")
        .bind(periodo).bind(trabajador).execute(&pool).await.unwrap().last_insert_rowid();
    let incapacidad: i64 = sqlx::query("INSERT INTO incapacidades (trabajador_id, tipo, dias) VALUES (?, 'ENFERMEDAD', 3)")
        .bind(trabajador).execute(&pool).await.unwrap().last_insert_rowid();
    let vacacion: i64 = sqlx::query("INSERT INTO vacaciones (trabajador_id, dias) VALUES (?, 5)")
        .bind(trabajador).execute(&pool).await.unwrap().last_insert_rowid();

    aplicar_0004(&pool).await;

    for (tabla, id) in [
        ("trabajadores", trabajador),
        ("periodos_nomina", periodo),
        ("recibos", recibo),
        ("incapacidades", incapacidad),
        ("vacaciones", vacacion),
    ] {
        let valor: i64 = sqlx::query_scalar(&format!("SELECT empresa_id FROM {} WHERE id = ?", tabla))
            .bind(id).fetch_one(&pool).await.unwrap();
        assert_eq!(valor, 1, "{} debió quedar en empresa_id=1", tabla);
    }

    let valor: String = sqlx::query_scalar("SELECT valor FROM config WHERE clave = 'empresa_activa'")
        .fetch_one(&pool).await.unwrap();
    assert_eq!(valor, "1");
    assert_eq!(empresa_activa_pool(&pool).await.unwrap(), 1);

    // El índice de RFC único debe existir en la BD migrada.
    let indices: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM sqlite_master WHERE type = 'index' AND name = 'ux_empresas_rfc'")
        .fetch_one(&pool).await.unwrap();
    assert_eq!(indices, 1);

    // Segundo escenario: BD sin empresas → la migración crea Empresa Default.
    let (raiz2, db2) = ruta_db("migracion_default_vacia");
    let pool2 = conectar(&db2).await;
    aplicar_base(&pool2).await;
    aplicar_0004(&pool2).await;
    let razon: String = sqlx::query_scalar("SELECT razon_social FROM empresas WHERE id = 1")
        .fetch_one(&pool2).await.unwrap();
    assert_eq!(razon, "Empresa Default");
    let rfc: String = sqlx::query_scalar("SELECT rfc FROM empresas WHERE id = 1")
        .fetch_one(&pool2).await.unwrap();
    assert_eq!(rfc, "XAXX010101000");

    drop(pool);
    drop(pool2);
    let _ = fs::remove_dir_all(&raiz);
    let _ = fs::remove_dir_all(&raiz2);
}
