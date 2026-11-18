// C4 — Todas las consultas de trabajadores se filtran por la empresa activa.
// El `empresa_id` que envía el formulario se IGNORA: se resuelve en servidor
// desde config(clave='empresa_activa').

use crate::commands::empresas::empresa_activa_pool;
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

const COLUMNAS: &str = "id, empresa_id, nombre, curp, rfc, nss, num_empleado, fecha_nacimiento, estado_civil, fecha_ingreso, tipo_contrato, departamento, puesto, salario_diario, periodicidad_pago, jornada, horas_semanales, infonavit_activo, infonavit_num_credito, infonavit_factor, infonavit_tipo_descuento, fecha_baja, tipo_baja";

fn db_url(app: &AppHandle) -> String {
    let dir = app.path().app_data_dir().unwrap();
    format!("sqlite:{}", dir.join("nomina.db").display())
}

async fn pool(app: &AppHandle) -> Result<SqlitePool, String> {
    SqlitePool::connect(&db_url(app)).await.map_err(|e| e.to_string())
}

fn fila_a_trabajador(r: sqlx::sqlite::SqliteRow) -> Trabajador {
    Trabajador {
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
    }
}

/// Guarda un trabajador **siempre** en la empresa activa, ignorando
/// `t.empresa_id` (que puede venir manipulado desde el cliente).
pub async fn guardar_trabajador_pool(p: &SqlitePool, t: &Trabajador) -> Result<i64, String> {
    let empresa_id = empresa_activa_pool(p).await?;
    if let Some(id) = t.id {
        sqlx::query(
            "UPDATE trabajadores SET empresa_id=?, nombre=?, curp=?, rfc=?, nss=?, num_empleado=?, fecha_nacimiento=?, estado_civil=?, fecha_ingreso=?, tipo_contrato=?, departamento=?, puesto=?, salario_diario=?, periodicidad_pago=?, jornada=?, horas_semanales=?, infonavit_activo=?, infonavit_num_credito=?, infonavit_factor=?, infonavit_tipo_descuento=?, fecha_baja=?, tipo_baja=? WHERE id=? AND empresa_id=?",
        )
        .bind(empresa_id).bind(&t.nombre).bind(&t.curp).bind(&t.rfc).bind(&t.nss)
        .bind(&t.num_empleado).bind(&t.fecha_nacimiento).bind(&t.estado_civil).bind(&t.fecha_ingreso)
        .bind(&t.tipo_contrato).bind(&t.departamento).bind(&t.puesto).bind(t.salario_diario)
        .bind(&t.periodicidad_pago).bind(&t.jornada).bind(t.horas_semanales)
        .bind(t.infonavit_activo as i64).bind(&t.infonavit_num_credito).bind(t.infonavit_factor)
        .bind(&t.infonavit_tipo_descuento).bind(&t.fecha_baja).bind(&t.tipo_baja)
        .bind(id).bind(empresa_id)
        .execute(p).await.map_err(|e| e.to_string())?;
        Ok(id)
    } else {
        let res = sqlx::query(
            "INSERT INTO trabajadores (empresa_id, nombre, curp, rfc, nss, num_empleado, fecha_nacimiento, estado_civil, fecha_ingreso, tipo_contrato, departamento, puesto, salario_diario, periodicidad_pago, jornada, horas_semanales, infonavit_activo, infonavit_num_credito, infonavit_factor, infonavit_tipo_descuento, fecha_baja, tipo_baja) VALUES (?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?)",
        )
        .bind(empresa_id).bind(&t.nombre).bind(&t.curp).bind(&t.rfc).bind(&t.nss)
        .bind(&t.num_empleado).bind(&t.fecha_nacimiento).bind(&t.estado_civil).bind(&t.fecha_ingreso)
        .bind(&t.tipo_contrato).bind(&t.departamento).bind(&t.puesto).bind(t.salario_diario)
        .bind(&t.periodicidad_pago).bind(&t.jornada).bind(t.horas_semanales)
        .bind(t.infonavit_activo as i64).bind(&t.infonavit_num_credito).bind(t.infonavit_factor)
        .bind(&t.infonavit_tipo_descuento).bind(&t.fecha_baja).bind(&t.tipo_baja)
        .execute(p).await.map_err(|e| e.to_string())?;
        Ok(res.last_insert_rowid())
    }
}

/// Lista únicamente los trabajadores de la empresa activa.
pub async fn listar_trabajadores_pool(p: &SqlitePool) -> Result<Vec<Trabajador>, String> {
    let empresa_id = empresa_activa_pool(p).await?;
    let sql = format!("SELECT {} FROM trabajadores WHERE empresa_id=? ORDER BY id", COLUMNAS);
    let rows = sqlx::query(&sql)
        .bind(empresa_id)
        .fetch_all(p)
        .await
        .map_err(|e| e.to_string())?;
    Ok(rows.into_iter().map(fila_a_trabajador).collect())
}

/// Obtiene un trabajador solo si pertenece a la empresa activa.
pub async fn obtener_trabajador_pool(p: &SqlitePool, id: i64) -> Result<Option<Trabajador>, String> {
    let empresa_id = empresa_activa_pool(p).await?;
    let sql = format!("SELECT {} FROM trabajadores WHERE id=? AND empresa_id=?", COLUMNAS);
    let row = sqlx::query(&sql)
        .bind(id)
        .bind(empresa_id)
        .fetch_optional(p)
        .await
        .map_err(|e| e.to_string())?;
    Ok(row.map(fila_a_trabajador))
}

#[tauri::command]
pub async fn guardar_trabajador(app: AppHandle, t: Trabajador) -> Result<i64, String> {
    let p = pool(&app).await?;
    guardar_trabajador_pool(&p, &t).await
}

#[tauri::command]
pub async fn listar_trabajadores(app: AppHandle, empresa_id: i64) -> Result<Vec<Trabajador>, String> {
    // El parámetro del cliente se conserva por compatibilidad con la firma
    // anterior, pero la empresa se resuelve en servidor (regla C4).
    let _ = empresa_id;
    let p = pool(&app).await?;
    listar_trabajadores_pool(&p).await
}

#[tauri::command]
pub async fn obtener_trabajador(app: AppHandle, id: i64) -> Result<Option<Trabajador>, String> {
    let p = pool(&app).await?;
    obtener_trabajador_pool(&p, id).await
}
