use serde::{Deserialize, Serialize};
use sqlx::{Row, SqlitePool};
use tauri::AppHandle;
use tauri::Manager;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Trabajador {
    pub id: Option<i64>,
    pub empresa_id: i64,
    pub nombre: String,
    pub curp: String,
    pub rfc: String,
    pub nss: String,
    pub num_empleado: String,
    pub fecha_nacimiento: String,
    pub estado_civil: String,
    pub fecha_ingreso: String,
    pub tipo_contrato: String,
    pub departamento: String,
    pub puesto: String,
    pub salario_diario: f64,
    pub periodicidad_pago: String,
    pub jornada: String,
    pub horas_semanales: f64,
    pub infonavit_activo: bool,
    pub infonavit_num_credito: Option<String>,
    pub infonavit_factor: f64,
    pub infonavit_tipo_descuento: String,
    pub fecha_baja: Option<String>,
    pub tipo_baja: Option<String>,
}

fn db_url(app: &AppHandle) -> String {
    let dir = app.path().app_data_dir().unwrap();
    format!("sqlite:{}", dir.join("nomina.db").display())
}

async fn pool(app: &AppHandle) -> Result<SqlitePool, String> {
    SqlitePool::connect(&db_url(app)).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn guardar_trabajador(app: AppHandle, t: Trabajador) -> Result<i64, String> {
    let p = pool(&app).await?;
    if let Some(id) = t.id {
        sqlx::query(
            "UPDATE trabajadores SET empresa_id=?, nombre=?, curp=?, rfc=?, nss=?, num_empleado=?, fecha_nacimiento=?, estado_civil=?, fecha_ingreso=?, tipo_contrato=?, departamento=?, puesto=?, salario_diario=?, periodicidad_pago=?, jornada=?, horas_semanales=?, infonavit_activo=?, infonavit_num_credito=?, infonavit_factor=?, infonavit_tipo_descuento=?, fecha_baja=?, tipo_baja=? WHERE id=?",
        )
        .bind(t.empresa_id).bind(&t.nombre).bind(&t.curp).bind(&t.rfc).bind(&t.nss)
        .bind(&t.num_empleado).bind(&t.fecha_nacimiento).bind(&t.estado_civil).bind(&t.fecha_ingreso)
        .bind(&t.tipo_contrato).bind(&t.departamento).bind(&t.puesto).bind(t.salario_diario)
        .bind(&t.periodicidad_pago).bind(&t.jornada).bind(t.horas_semanales)
        .bind(t.infonavit_activo as i64).bind(&t.infonavit_num_credito).bind(t.infonavit_factor)
        .bind(&t.infonavit_tipo_descuento).bind(&t.fecha_baja).bind(&t.tipo_baja).bind(id)
        .execute(&p).await.map_err(|e| e.to_string())?;
        Ok(id)
    } else {
        let res = sqlx::query(
            "INSERT INTO trabajadores (empresa_id, nombre, curp, rfc, nss, num_empleado, fecha_nacimiento, estado_civil, fecha_ingreso, tipo_contrato, departamento, puesto, salario_diario, periodicidad_pago, jornada, horas_semanales, infonavit_activo, infonavit_num_credito, infonavit_factor, infonavit_tipo_descuento, fecha_baja, tipo_baja) VALUES (?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?)",
        )
        .bind(t.empresa_id).bind(&t.nombre).bind(&t.curp).bind(&t.rfc).bind(&t.nss)
        .bind(&t.num_empleado).bind(&t.fecha_nacimiento).bind(&t.estado_civil).bind(&t.fecha_ingreso)
        .bind(&t.tipo_contrato).bind(&t.departamento).bind(&t.puesto).bind(t.salario_diario)
        .bind(&t.periodicidad_pago).bind(&t.jornada).bind(t.horas_semanales)
        .bind(t.infonavit_activo as i64).bind(&t.infonavit_num_credito).bind(t.infonavit_factor)
        .bind(&t.infonavit_tipo_descuento).bind(&t.fecha_baja).bind(&t.tipo_baja)
        .execute(&p).await.map_err(|e| e.to_string())?;
        Ok(res.last_insert_rowid())
    }
}

#[tauri::command]
pub async fn listar_trabajadores(app: AppHandle, empresa_id: i64) -> Result<Vec<Trabajador>, String> {
    let p = pool(&app).await?;
    let rows = sqlx::query(
        "SELECT id, empresa_id, nombre, curp, rfc, nss, num_empleado, fecha_nacimiento, estado_civil, fecha_ingreso, tipo_contrato, departamento, puesto, salario_diario, periodicidad_pago, jornada, horas_semanales, infonavit_activo, infonavit_num_credito, infonavit_factor, infonavit_tipo_descuento, fecha_baja, tipo_baja FROM trabajadores WHERE empresa_id=?",
    )
    .bind(empresa_id).fetch_all(&p).await.map_err(|e| e.to_string())?;
    let mut out = vec![];
    for r in rows {
        out.push(Trabajador {
            id: Some(r.get("id")),
            empresa_id: r.get("empresa_id"),
            nombre: r.get("nombre"),
            curp: r.get("curp"),
            rfc: r.get("rfc"),
            nss: r.get("nss"),
            num_empleado: r.get("num_empleado"),
            fecha_nacimiento: r.get("fecha_nacimiento"),
            estado_civil: r.get("estado_civil"),
            fecha_ingreso: r.get("fecha_ingreso"),
            tipo_contrato: r.get("tipo_contrato"),
            departamento: r.get("departamento"),
            puesto: r.get("puesto"),
            salario_diario: r.get("salario_diario"),
            periodicidad_pago: r.get("periodicidad_pago"),
            jornada: r.get("jornada"),
            horas_semanales: r.get("horas_semanales"),
            infonavit_activo: r.get::<i64, _>("infonavit_activo") != 0,
            infonavit_num_credito: r.get("infonavit_num_credito"),
            infonavit_factor: r.get("infonavit_factor"),
            infonavit_tipo_descuento: r.get("infonavit_tipo_descuento"),
            fecha_baja: r.get("fecha_baja"),
            tipo_baja: r.get("tipo_baja"),
        });
    }
    Ok(out)
}

#[tauri::command]
pub async fn obtener_trabajador(app: AppHandle, id: i64) -> Result<Option<Trabajador>, String> {
    let p = pool(&app).await?;
    let row = sqlx::query(
        "SELECT id, empresa_id, nombre, curp, rfc, nss, num_empleado, fecha_nacimiento, estado_civil, fecha_ingreso, tipo_contrato, departamento, puesto, salario_diario, periodicidad_pago, jornada, horas_semanales, infonavit_activo, infonavit_num_credito, infonavit_factor, infonavit_tipo_descuento, fecha_baja, tipo_baja FROM trabajadores WHERE id=?",
    )
    .bind(id).fetch_optional(&p).await.map_err(|e| e.to_string())?;
    Ok(row.map(|r| Trabajador {
        id: Some(r.get("id")),
        empresa_id: r.get("empresa_id"),
        nombre: r.get("nombre"),
        curp: r.get("curp"),
        rfc: r.get("rfc"),
        nss: r.get("nss"),
        num_empleado: r.get("num_empleado"),
        fecha_nacimiento: r.get("fecha_nacimiento"),
        estado_civil: r.get("estado_civil"),
        fecha_ingreso: r.get("fecha_ingreso"),
        tipo_contrato: r.get("tipo_contrato"),
        departamento: r.get("departamento"),
        puesto: r.get("puesto"),
        salario_diario: r.get("salario_diario"),
        periodicidad_pago: r.get("periodicidad_pago"),
        jornada: r.get("jornada"),
        horas_semanales: r.get("horas_semanales"),
        infonavit_activo: r.get::<i64, _>("infonavit_activo") != 0,
        infonavit_num_credito: r.get("infonavit_num_credito"),
        infonavit_factor: r.get("infonavit_factor"),
        infonavit_tipo_descuento: r.get("infonavit_tipo_descuento"),
        fecha_baja: r.get("fecha_baja"),
        tipo_baja: r.get("tipo_baja"),
    }))
}