use crate::calc::{horas_extra, imss, infonavit, isn, isr, sar, sdi};
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
    // Bloque B: desglose de tiempo extraordinario (LFT 66/68, LISR 93 I).
    pub horas_extra_dobles: f64,
    pub horas_extra_triples: f64,
    pub horas_extra_importe: f64,
    pub horas_extra_exento: f64,
    pub horas_extra_gravado: f64,
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

/// Captura de horas extraordinarias del periodo. `semanas` determina el tope
/// semanal de la exencion del art. 93 LISR; `salario_minimo` activa el
/// supuesto de exencion del 100% de las horas dentro del limite laboral.
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct HorasExtraInput {
    #[serde(default)]
    pub horas: f64,
    #[serde(default)]
    pub dias: i64,
    #[serde(default)]
    pub semanas: f64,
    #[serde(default)]
    pub jornada: Option<String>,
    #[serde(default)]
    pub salario_minimo: Option<bool>,
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
    horas_extra: Option<HorasExtraInput>,
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

    // Bloque B: los parametros de horas extra se leen de la fila vigente por
    // fecha_pago. Si no existe fila, falla explicitamente: no se inventan tasas.
    let he_json = cargar_param_vigente(&app, "horas_extra", &fecha_pago).await
        .map_err(|e| format!("sin parametros de horas extra vigentes para {}: {}", fecha_pago, e))?;
    let he_params: horas_extra::ParametrosHorasExtra = serde_json::from_str(&he_json)
        .map_err(|e| format!("parametros de horas extra invalidos: {}", e))?;

    let he = match horas_extra.as_ref() {
        Some(h) if h.horas > 0.0 => {
            let horas_jornada = horas_extra::horas_jornada_diaria(h.jornada.as_deref(), &he_params)?;
            let desglose = horas_extra::desglosar_horas_extra(
                h.horas, salario_diario, horas_jornada, &he_params,
            )?;
            let semanas = if h.semanas > 0.0 { h.semanas } else { 1.0 };
            let exencion = horas_extra::calcular_exencion(
                desglose.importe_dobles,
                desglose.importe_triples,
                h.salario_minimo.unwrap_or(false),
                uma_diaria,
                semanas,
                &he_params,
            );
            Some((desglose, exencion))
        }
        _ => None,
    };

    let dias_f = dias as f64;
    // SDI base; las horas extra son salario variable que integra el SBC del
    // periodo (LSS art. 27) y se topa a 25 UMA (LSS art. 28).
    let sbc_base = sdi::calcular_sdi(salario_diario, 15.0, 12.0, 0.25, uma_diaria);
    let sbc_diario = match &he {
        Some((d, _)) => horas_extra::integrar_en_sbc(sbc_base, d.importe_total, dias_f, uma_diaria)?,
        None => sbc_base,
    };
    let sbc_total = sbc_diario * dias_f;

    // IMSS
    let cuotas = imss::calcular_cuotas_imss(sbc_diario, dias_f, uma_diaria, prima_rt, 0.0, sm_general);

    // INFONAVIT
    let info = infonavit::calcular_infonavit(sbc_total, salario_diario, dias_f, infonavit_factor, &infonavit_tipo, infonavit_activo, uma_diaria, sm_general);

    // SAR
    let sar_aport = sar::calcular_sar(sbc_total);

    // ISR: solo la parte gravada de las horas extra se suma a la base (LISR 93 I).
    let ingreso_ordinario = salario_diario * dias_f;
    let importe_he = he.as_ref().map(|(d, _)| d.importe_total).unwrap_or(0.0);
    let exento_he = he.as_ref().map(|(_, e)| e.exento).unwrap_or(0.0);
    let gravado_he = he.as_ref().map(|(_, e)| e.gravado).unwrap_or(0.0);
    let ingreso_gravado = ingreso_ordinario + gravado_he;
    let (isr_neto, _bruto, _sub) = isr::calcular_isr_neto(ingreso_gravado, &tarifa, sub_max, sub_tope);

    // ISN: opera sobre el total percibido, incluida la parte exenta de ISR.
    let total_percepciones = ingreso_ordinario + importe_he;
    let isn_val = isn::calcular_isn(total_percepciones, tasa_isn);

    let total_deducciones = isr_neto + cuotas.obrero + info.descuento_trabajador;
    let neto = total_percepciones - total_deducciones;
    let costo_patron = cuotas.patronal + info.aportacion_patronal + sar_aport + isn_val;

    Ok(ReciboCalculado {
        trabajador_id,
        sueldo_bruto: total_percepciones,
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
        horas_extra_dobles: he.as_ref().map(|(d, _)| d.horas_dobles).unwrap_or(0.0),
        horas_extra_triples: he.as_ref().map(|(d, _)| d.horas_triples).unwrap_or(0.0),
        horas_extra_importe: importe_he,
        horas_extra_exento: exento_he,
        horas_extra_gravado: gravado_he,
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
        "INSERT INTO recibos (periodo_id, trabajador_id, sueldo_bruto, isr, imss_obrero, infonavit_desc, otras_deducciones, total_percepciones, total_deducciones, neto, cuota_patronal_imss, aportacion_infonavit, aportacion_sar, isn, costo_total_patron, horas_extra_dobles, horas_extra_triples, horas_extra_importe, horas_extra_exento, horas_extra_gravado) VALUES (?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?)",
    )
    .bind(periodo_id).bind(r.trabajador_id).bind(r.sueldo_bruto).bind(r.isr).bind(r.imss_obrero)
    .bind(r.infonavit_desc).bind(r.otras_deducciones).bind(r.total_percepciones).bind(r.total_deducciones)
    .bind(r.neto).bind(r.cuota_patronal_imss).bind(r.aportacion_infonavit).bind(r.aportacion_sar)
    .bind(r.isn).bind(r.costo_total_patron)
    .bind(r.horas_extra_dobles).bind(r.horas_extra_triples).bind(r.horas_extra_importe)
    .bind(r.horas_extra_exento).bind(r.horas_extra_gravado)
    .execute(&p).await.map_err(|e| e.to_string())?;
    Ok(res.last_insert_rowid())
}