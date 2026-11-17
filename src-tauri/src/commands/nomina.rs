use crate::calc::{imss, infonavit, isn, isr, sar, sdi};
use crate::calc::isr::TarifaBracket;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use tauri::AppHandle;
use tauri::Manager;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ReciboCalculado {
    pub trabajador_id: i64,
    pub sueldo_bruto: f64,
    pub isr: f64,
    pub imss_obrero: f64,
    pub infonavit_desc: f64,
    pub otras_deducciones: f64,
    pub total_percepciones: f64,
    pub total_deducciones: f64,
    pub neto: f64,
    pub cuota_patronal_imss: f64,
    pub aportacion_infonavit: f64,
    pub aportacion_sar: f64,
    pub isn: f64,
    pub costo_total_patron: f64,
    pub sbc_diario: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PeriodoNomina {
    pub id: Option<i64>,
    pub empresa_id: i64,
    pub tipo: String,
    pub fecha_inicio: String,
    pub fecha_fin: String,
    pub fecha_pago: String,
    pub estado: String,
    pub dias: i64,
}

fn db_url(app: &AppHandle) -> String {
    let dir = app.path().app_data_dir().unwrap();
    format!("sqlite:{}", dir.join("nomina.db").display())
}

async fn pool(app: &AppHandle) -> Result<SqlitePool, String> {
    SqlitePool::connect(&db_url(app)).await.map_err(|e| e.to_string())
}

// C6: selecciona la fila vigente segun fecha_pago (enero vs febrero 2026).
async fn cargar_param_vigente(app: &AppHandle, tipo: &str, fecha_pago: &str) -> Result<String, String> {
    let p = pool(app).await?;
    let row: (String,) = sqlx::query_as(
        "SELECT datos_json FROM tablas_fiscales WHERE tipo=? AND vigencia_inicio<=? AND vigencia_fin>=? ORDER BY vigencia_inicio DESC LIMIT 1",
    )
    .bind(tipo).bind(fecha_pago).bind(fecha_pago)
    .fetch_one(&p).await.map_err(|e| e.to_string())?;
    Ok(row.0)
}

#[tauri::command]
pub async fn calcular_recibo(
    app: AppHandle,
    trabajador_id: i64,
    empresa_id: i64,
    dias: i64,
    periodicidad: String,
    fecha_pago: String,
) -> Result<ReciboCalculado, String> {
    let p = pool(&app).await?;
    // trabajador
    let trow = sqlx::query_as::<_, (i64, f64, String, i64, f64, String)>(
        "SELECT id, salario_diario, periodicidad_pago, infonavit_activo, infonavit_factor, infonavit_tipo_descuento FROM trabajadores WHERE id=?",
    )
    .bind(trabajador_id).fetch_one(&p).await.map_err(|e| e.to_string())?;
    let salario_diario = trow.1;
    let infonavit_activo = trow.3 != 0;
    let infonavit_factor = trow.4;
    let infonavit_tipo = trow.5.clone();

    // empresa
    let erow = sqlx::query_as::<_, (f64, f64, String)>(
        "SELECT prima_riesgo_trabajo, tasa_isn, estado_isn FROM empresas WHERE id=?",
    )
    .bind(empresa_id).fetch_one(&p).await.map_err(|e| e.to_string())?;
    let prima_rt = erow.0;
    let tasa_isn = erow.1;

    // params
    let uma_json = cargar_param_vigente(&app, "uma", &fecha_pago).await.unwrap_or_else(|_| r#"{"diaria":117.31,"mensual":3566.22,"anual":42794.64}"#.to_string());
    let uma: serde_json::Value = serde_json::from_str(&uma_json).unwrap_or_default();
    let uma_diaria = uma.get("diaria").and_then(|v| v.as_f64()).unwrap_or(117.31);

    let sm_json = cargar_param_vigente(&app, "salario_minimo", &fecha_pago).await.unwrap_or_else(|_| r#"{"general":278.80,"frontera":419.88}"#.to_string());
    let sm: serde_json::Value = serde_json::from_str(&sm_json).unwrap_or_default();
    let sm_general = sm.get("general").and_then(|v| v.as_f64()).unwrap_or(278.80);

    // tarifa isr
    let tipo_tarifa = match periodicidad.as_str() {
        "quincenal" => "tarifa_isr_quincenal",
        "semanal" => "tarifa_isr_semanal",
        _ => "tarifa_isr_mensual",
    };
    let tarifa_json = cargar_param_vigente(&app, tipo_tarifa, &fecha_pago).await.unwrap_or_else(|_| "[]".to_string());
    let tarifa: Vec<TarifaBracket> = serde_json::from_str(&tarifa_json).unwrap_or_default();
    // subsidio
    let sub_json = cargar_param_vigente(&app, "subsidio_empleo", &fecha_pago).await.unwrap_or_else(|_| r#"{"maximo":535.65,"tope":11492.66}"#.to_string());
    let sub: serde_json::Value = serde_json::from_str(&sub_json).unwrap_or_default();
    let sub_max = sub.get("maximo").and_then(|v| v.as_f64()).unwrap_or(535.65);
    let sub_tope = sub.get("tope").and_then(|v| v.as_f64()).unwrap_or(11492.66);

    let dias_f = dias as f64;
    // SDI
    let sbc_diario = sdi::calcular_sdi(salario_diario, 15.0, 12.0, 0.25, uma_diaria);
    let sbc_total = sbc_diario * dias_f;

    // IMSS
    let cuotas = imss::calcular_cuotas_imss(sbc_diario, dias_f, uma_diaria, prima_rt, 0.0, sm_general);

    // INFONAVIT
    let info = infonavit::calcular_infonavit(sbc_total, salario_diario, dias_f, infonavit_factor, &infonavit_tipo, infonavit_activo, uma_diaria, sm_general);

    // SAR
    let sar_aport = sar::calcular_sar(sbc_total);

    // ISR
    let ingreso_periodo = salario_diario * dias_f;
    let (isr_neto, _bruto, _sub) = isr::calcular_isr_neto(ingreso_periodo, &tarifa, sub_max, sub_tope);

    // ISN
    let isn_val = isn::calcular_isn(ingreso_periodo, tasa_isn);

    let total_percepciones = ingreso_periodo;
    let total_deducciones = isr_neto + cuotas.obrero + info.descuento_trabajador;
    let neto = total_percepciones - total_deducciones;
    let costo_patron = cuotas.patronal + info.aportacion_patronal + sar_aport + isn_val;

    Ok(ReciboCalculado {
        trabajador_id,
        sueldo_bruto: ingreso_periodo,
        isr: isr_neto,
        imss_obrero: cuotas.obrero,
        infonavit_desc: info.descuento_trabajador,
        otras_deducciones: 0.0,
        total_percepciones,
        total_deducciones,
        neto,
        cuota_patronal_imss: cuotas.patronal,
        aportacion_infonavit: info.aportacion_patronal,
        aportacion_sar: sar_aport,
        isn: isn_val,
        costo_total_patron: costo_patron,
        sbc_diario,
    })
}

#[tauri::command]
pub async fn guardar_periodo(app: AppHandle, periodo: PeriodoNomina) -> Result<i64, String> {
    let p = pool(&app).await?;
    let res = sqlx::query(
        "INSERT INTO periodos_nomina (empresa_id, tipo, fecha_inicio, fecha_fin, fecha_pago, estado, dias) VALUES (?,?,?,?,?,?,?)",
    )
    .bind(periodo.empresa_id).bind(&periodo.tipo).bind(&periodo.fecha_inicio)
    .bind(&periodo.fecha_fin).bind(&periodo.fecha_pago).bind(&periodo.estado).bind(periodo.dias)
    .execute(&p).await.map_err(|e| e.to_string())?;
    Ok(res.last_insert_rowid())
}

#[tauri::command]
pub async fn listar_periodos(app: AppHandle, empresa_id: i64) -> Result<Vec<PeriodoNomina>, String> {
    let p = pool(&app).await?;
    let rows = sqlx::query_as::<_, (i64, i64, String, String, String, String, String, i64)>(
        "SELECT id, empresa_id, tipo, fecha_inicio, fecha_fin, fecha_pago, estado, dias FROM periodos_nomina WHERE empresa_id=?",
    )
    .bind(empresa_id).fetch_all(&p).await.map_err(|e| e.to_string())?;
    Ok(rows.into_iter().map(|r| PeriodoNomina {
        id: Some(r.0), empresa_id: r.1, tipo: r.2, fecha_inicio: r.3, fecha_fin: r.4,
        fecha_pago: r.5, estado: r.6, dias: r.7,
    }).collect())
}

#[tauri::command]
pub async fn guardar_recibo(app: AppHandle, periodo_id: i64, r: ReciboCalculado) -> Result<i64, String> {
    let p = pool(&app).await?;
    let res = sqlx::query(
        "INSERT INTO recibos (periodo_id, trabajador_id, sueldo_bruto, isr, imss_obrero, infonavit_desc, otras_deducciones, total_percepciones, total_deducciones, neto, cuota_patronal_imss, aportacion_infonavit, aportacion_sar, isn, costo_total_patron) VALUES (?,?,?,?,?,?,?,?,?,?,?,?,?,?,?)",
    )
    .bind(periodo_id).bind(r.trabajador_id).bind(r.sueldo_bruto).bind(r.isr).bind(r.imss_obrero)
    .bind(r.infonavit_desc).bind(r.otras_deducciones).bind(r.total_percepciones).bind(r.total_deducciones)
    .bind(r.neto).bind(r.cuota_patronal_imss).bind(r.aportacion_infonavit).bind(r.aportacion_sar)
    .bind(r.isn).bind(r.costo_total_patron)
    .execute(&p).await.map_err(|e| e.to_string())?;
    Ok(res.last_insert_rowid())
}