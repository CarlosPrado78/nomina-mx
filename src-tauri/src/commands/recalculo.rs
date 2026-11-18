// C5 — Recálculo retroactivo de SBC y recibos.
//
// Fuentes:
//   - CFF art. 28: conservación de registros; toda revisión queda en audit_log.
//   - LSS arts. 27 y 28: SBC y tope de 25 UMA.
//   - LISR arts. 96 y 99: ISR y subsidio al empleo.
//   - RMF Anexos 8 y 11: tarifas y parámetros con vigencia.
//   - CFDI 4.0 / Nómina 1.2: un comprobante timbrado es inmutable. El ajuste
//     con TipoDeComprobante="E" queda pendiente para v1.1.
//
// Regla central: los parámetros se resuelven con la fecha de pago ORIGINAL del
// periodo, nunca con la fecha actual. Un periodo de enero 2026 usa UMA 113.14
// y subsidio 536.21 aunque se recalcule en noviembre 2026.

use crate::calc::isr::{self, TarifaBracket};
use crate::calc::{horas_extra, imss, infonavit, isn, sar, sdi};
use crate::commands::auditoria::sincronizar_usuario_pool;
use crate::commands::empresas::empresa_activa_pool;
use crate::commands::nomina::{HorasExtraInput, PeriodoNomina, ReciboCalculado};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use tauri::AppHandle;
use tauri::Manager;

/// Motivo usado por la ruta individual: la firma pública no recibe motivo,
/// pero el registro de auditoría SIEMPRE lleva uno no vacío.
const MOTIVO_INDIVIDUAL: &str = "Recálculo individual solicitado desde la UI";

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Diferencia {
    pub trabajador_id: i64,
    pub nombre: String,
    pub percepciones_antes: f64,
    pub percepciones_despues: f64,
    pub deducciones_antes: f64,
    pub deducciones_despues: f64,
    pub neto_antes: f64,
    pub neto_despues: f64,
    pub delta_neto: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct ResultadoRecalculo {
    /// Recibos efectivamente actualizados (excluye los que ya tienen UUID).
    pub recibos_recalculados: usize,
    /// Cambios detectados por trabajador, incluidos los recibos timbrados
    /// (se reportan, pero NO se modifican).
    pub diferencias: Vec<Diferencia>,
}

fn db_url(app: &AppHandle) -> String {
    let dir = app.path().app_data_dir().unwrap();
    format!("sqlite:{}", dir.join("nomina.db").display())
}

async fn pool(app: &AppHandle) -> Result<SqlitePool, String> {
    SqlitePool::connect(&db_url(app)).await.map_err(|e| e.to_string())
}

fn validar_motivo(motivo: &str) -> Result<String, String> {
    let limpio = motivo.trim().to_string();
    if limpio.is_empty() {
        Err("el motivo del recálculo es obligatorio".to_string())
    } else {
        Ok(limpio)
    }
}

/// Resuelve la fila vigente de un parámetro a una fecha concreta.
async fn cargar_param_vigente_pool(
    p: &SqlitePool,
    tipo: &str,
    fecha_pago: &str,
) -> Result<String, String> {
    let fila: (String,) = sqlx::query_as(
        "SELECT datos_json FROM tablas_fiscales \
         WHERE tipo=? AND vigencia_inicio<=? AND vigencia_fin>=? \
         ORDER BY vigencia_inicio DESC LIMIT 1",
    )
    .bind(tipo)
    .bind(fecha_pago)
    .bind(fecha_pago)
    .fetch_one(p)
    .await
    .map_err(|e| format!("sin parámetro '{}' vigente para {}: {}", tipo, fecha_pago, e))?;
    Ok(fila.0)
}

/// UMA, subsidio máximo y tope vigentes a la fecha de pago original.
async fn resolver_parametros(
    p: &SqlitePool,
    fecha_pago: &str,
) -> Result<(f64, f64, f64), String> {
    let uma_json = cargar_param_vigente_pool(p, "uma", fecha_pago).await?;
    let uma: serde_json::Value = serde_json::from_str(&uma_json)
        .map_err(|e| format!("parámetros UMA inválidos: {}", e))?;
    let uma_diaria = uma
        .get("diaria")
        .and_then(|v| v.as_f64())
        .ok_or_else(|| "el parámetro UMA no tiene 'diaria'".to_string())?;

    let sub_json = cargar_param_vigente_pool(p, "subsidio_empleo", fecha_pago).await?;
    let sub: serde_json::Value = serde_json::from_str(&sub_json)
        .map_err(|e| format!("parámetros de subsidio inválidos: {}", e))?;
    let sub_max = sub
        .get("maximo")
        .and_then(|v| v.as_f64())
        .ok_or_else(|| "el subsidio no tiene 'maximo'".to_string())?;
    let sub_tope = sub
        .get("tope")
        .and_then(|v| v.as_f64())
        .unwrap_or(11492.66);

    Ok((uma_diaria, sub_max, sub_tope))
}

/// Recalcula un recibo con los parámetros vigentes a `fecha_pago`.
/// Es público para que la evidencia pueda verificar cifras exactas sin depender
/// de los comandos Tauri.
pub async fn calcular_recibo_historico_pool(
    p: &SqlitePool,
    empresa_id: i64,
    trabajador_id: i64,
    dias: i64,
    fecha_pago: &str,
    horas_extra_input: Option<HorasExtraInput>,
) -> Result<ReciboCalculado, String> {
    // Trabajador (misma regla C4: debe pertenecer a la empresa activa).
    let trow = sqlx::query_as::<_, (f64, String, i64, f64, String)>(
        "SELECT salario_diario, periodicidad_pago, infonavit_activo, \
                infonavit_factor, infonavit_tipo_descuento \
         FROM trabajadores WHERE id=? AND empresa_id=?",
    )
    .bind(trabajador_id)
    .bind(empresa_id)
    .fetch_one(p)
    .await
    .map_err(|e| format!("el trabajador no pertenece a la empresa activa: {}", e))?;
    let salario_diario = trow.0;
    let periodicidad = trow.1;
    let infonavit_activo = trow.2 != 0;
    let infonavit_factor = trow.3;
    let infonavit_tipo = trow.4;

    let erow = sqlx::query_as::<_, (f64, f64)>(
        "SELECT prima_riesgo_trabajo, tasa_isn FROM empresas WHERE id=?",
    )
    .bind(empresa_id)
    .fetch_one(p)
    .await
    .map_err(|e| e.to_string())?;
    let prima_rt = erow.0;
    let tasa_isn = erow.1;

    let (uma_diaria, sub_max, sub_tope) = resolver_parametros(p, fecha_pago).await?;

    let sm_json = cargar_param_vigente_pool(p, "salario_minimo", fecha_pago).await?;
    let sm: serde_json::Value =
        serde_json::from_str(&sm_json).map_err(|e| format!("salario mínimo inválido: {}", e))?;
    let sm_general = sm
        .get("general")
        .and_then(|v| v.as_f64())
        .ok_or_else(|| "el salario mínimo no tiene 'general'".to_string())?;

    let tipo_tarifa = match periodicidad.as_str() {
        "quincenal" => "tarifa_isr_quincenal",
        "semanal" => "tarifa_isr_semanal",
        _ => "tarifa_isr_mensual",
    };
    let tarifa_json = cargar_param_vigente_pool(p, tipo_tarifa, fecha_pago).await?;
    let tarifa: Vec<TarifaBracket> = serde_json::from_str(&tarifa_json)
        .map_err(|e| format!("tarifa ISR inválida: {}", e))?;

    let he_json = cargar_param_vigente_pool(p, "horas_extra", fecha_pago).await?;
    let he_params: horas_extra::ParametrosHorasExtra = serde_json::from_str(&he_json)
        .map_err(|e| format!("parámetros de horas extra inválidos: {}", e))?;

    let he = match horas_extra_input.as_ref() {
        Some(h) if h.horas > 0.0 => {
            let horas_jornada = horas_extra::horas_jornada_diaria(h.jornada.as_deref(), &he_params)?;
            let desglose = horas_extra::desglosar_horas_extra(
                h.horas,
                salario_diario,
                horas_jornada,
                &he_params,
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
    let sbc_base = sdi::calcular_sdi(salario_diario, 15.0, 12.0, 0.25, uma_diaria);
    let sbc_diario = match &he {
        Some((d, _)) => horas_extra::integrar_en_sbc(sbc_base, d.importe_total, dias_f, uma_diaria)?,
        None => sbc_base,
    };
    let sbc_total = sbc_diario * dias_f;

    let cuotas = imss::calcular_cuotas_imss(
        sbc_diario,
        dias_f,
        uma_diaria,
        prima_rt,
        0.0,
        sm_general,
    );
    let info = infonavit::calcular_infonavit(
        sbc_total,
        salario_diario,
        dias_f,
        infonavit_factor,
        &infonavit_tipo,
        infonavit_activo,
        uma_diaria,
        sm_general,
    );
    let sar_aport = sar::calcular_sar(sbc_total);

    let importe_he = he.as_ref().map(|(d, _)| d.importe_total).unwrap_or(0.0);
    let exento_he = he.as_ref().map(|(_, e)| e.exento).unwrap_or(0.0);
    let gravado_he = he.as_ref().map(|(_, e)| e.gravado).unwrap_or(0.0);
    let ingreso_gravado = salario_diario * dias_f + gravado_he;
    let (isr_neto, _bruto, _sub) =
        isr::calcular_isr_neto(ingreso_gravado, &tarifa, sub_max, sub_tope);

    let total_percepciones = salario_diario * dias_f + importe_he;
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

async fn cargar_periodo_pool(p: &SqlitePool, periodo_id: i64) -> Result<PeriodoNomina, String> {
    let empresa_id = empresa_activa_pool(p).await?;

    let fila = sqlx::query_as::<_, (i64, i64, String, String, String, String, String, i64)>(
        "SELECT id, empresa_id, tipo, fecha_inicio, fecha_fin, fecha_pago, estado, dias \
         FROM periodos_nomina WHERE id=? AND empresa_id=?",
    )
    .bind(periodo_id)
    .bind(empresa_id)
    .fetch_optional(p)
    .await
    .map_err(|e| e.to_string())?
    .ok_or_else(|| format!("el periodo {} no existe en la empresa activa", periodo_id))?;

    Ok(PeriodoNomina {
        id: Some(fila.0),
        empresa_id: fila.1,
        tipo: fila.2,
        fecha_inicio: fila.3,
        fecha_fin: fila.4,
        fecha_pago: fila.5,
        estado: fila.6,
        dias: fila.7,
    })
}

struct ReciboOriginal {
    id: i64,
    trabajador_id: i64,
    nombre: String,
    total_percepciones: f64,
    total_deducciones: f64,
    neto: f64,
    horas_extra_dobles: f64,
    horas_extra_triples: f64,
    uuid_cfdi: Option<String>,
}

async fn cargar_recibos_pool(
    p: &SqlitePool,
    periodo_id: i64,
) -> Result<Vec<ReciboOriginal>, String> {
    // Se mantienen 9 columnas para no exceder el máximo de 16 elementos de
    // `FromRow` para tuplas en sqlx. El resto de importes se recalculan.
    let filas = sqlx::query_as::<_, (
        i64, i64, String, f64, f64, f64, f64, f64, Option<String>,
    )>(
        "SELECT r.id, r.trabajador_id, t.nombre, \
                COALESCE(r.total_percepciones,0), COALESCE(r.total_deducciones,0), \
                COALESCE(r.neto,0), \
                COALESCE(r.horas_extra_dobles,0), COALESCE(r.horas_extra_triples,0), \
                r.uuid_cfdi \
         FROM recibos r JOIN trabajadores t ON t.id = r.trabajador_id \
         WHERE r.periodo_id=? ORDER BY r.id",
    )
    .bind(periodo_id)
    .fetch_all(p)
    .await
    .map_err(|e| e.to_string())?;

    Ok(filas
        .into_iter()
        .map(|f| ReciboOriginal {
            id: f.0,
            trabajador_id: f.1,
            nombre: f.2,
            total_percepciones: f.3,
            total_deducciones: f.4,
            neto: f.5,
            horas_extra_dobles: f.6,
            horas_extra_triples: f.7,
            uuid_cfdi: f.8,
        })
        .collect())
}

/// Las horas extra del recibo original se reconstruyen como tiempo total;
/// con los parámetros 2026 el reparto dobles/triples es determinista.
fn horas_extra_desde_recibo(r: &ReciboOriginal) -> Option<HorasExtraInput> {
    let total = r.horas_extra_dobles + r.horas_extra_triples;
    if total <= 0.0 {
        return None;
    }
    Some(HorasExtraInput {
        horas: total,
        dias: 1,
        semanas: 1.0,
        jornada: Some("diurna".to_string()),
        salario_minimo: Some(false),
    })
}

fn cambio(antes: f64, despues: f64) -> bool {
    (antes - despues).abs() >= 0.005
}

fn hubo_cambio(r: &ReciboOriginal, c: &ReciboCalculado) -> bool {
    cambio(r.total_percepciones, c.total_percepciones)
        || cambio(r.total_deducciones, c.total_deducciones)
        || cambio(r.neto, c.neto)
}

fn construir_diferencia(r: &ReciboOriginal, c: &ReciboCalculado) -> Diferencia {
    Diferencia {
        trabajador_id: r.trabajador_id,
        nombre: r.nombre.clone(),
        percepciones_antes: r.total_percepciones,
        percepciones_despues: c.total_percepciones,
        deducciones_antes: r.total_deducciones,
        deducciones_despues: c.total_deducciones,
        neto_antes: r.neto,
        neto_despues: c.neto,
        delta_neto: c.neto - r.neto,
    }
}

/// Núcleo compartido por los dos comandos. `solo_trabajador` limita el
/// recálculo a un recibo; `None` procesa todo el periodo.
async fn recalcular_pool(
    p: &SqlitePool,
    periodo_id: i64,
    motivo: &str,
    solo_trabajador: Option<i64>,
) -> Result<ResultadoRecalculo, String> {
    let motivo = validar_motivo(motivo)?;
    let periodo = cargar_periodo_pool(p, periodo_id).await?;
    let originales = cargar_recibos_pool(p, periodo_id).await?;

    let mut diferencias = Vec::new();
    let mut actualizaciones: Vec<(i64, ReciboCalculado)> = Vec::new();
    let mut neto_antes_total = 0.0_f64;
    let mut delta_neto_total = 0.0_f64;

    for original in originales {
        if let Some(id) = solo_trabajador {
            if original.trabajador_id != id {
                continue;
            }
        }
        neto_antes_total += original.neto;

        let calculado = calcular_recibo_historico_pool(
            p,
            periodo.empresa_id,
            original.trabajador_id,
            periodo.dias,
            &periodo.fecha_pago,
            horas_extra_desde_recibo(&original),
        )
        .await?;

        if hubo_cambio(&original, &calculado) {
            let diferencia = construir_diferencia(&original, &calculado);
            delta_neto_total += diferencia.delta_neto;
            diferencias.push(diferencia);

            // Un CFDI timbrado es inmutable: se reporta, pero no se UPDATE.
            if original.uuid_cfdi.is_none() {
                actualizaciones.push((original.id, calculado));
            }
        }
    }

    let (uma_diaria, subsidio_maximo, subsidio_tope) =
        resolver_parametros(p, &periodo.fecha_pago).await?;
    let usuario = sincronizar_usuario_pool(p).await?;

    let mut tx = p.begin().await.map_err(|e| e.to_string())?;
    let mut recibos_recalculados = 0_usize;
    for (id, c) in &actualizaciones {
        let res = sqlx::query(
            "UPDATE recibos SET sueldo_bruto=?, isr=?, imss_obrero=?, infonavit_desc=?, \
             otras_deducciones=?, total_percepciones=?, total_deducciones=?, neto=?, \
             cuota_patronal_imss=?, aportacion_infonavit=?, aportacion_sar=?, isn=?, \
             costo_total_patron=?, horas_extra_dobles=?, horas_extra_triples=?, \
             horas_extra_importe=?, horas_extra_exento=?, horas_extra_gravado=? \
             WHERE id=? AND uuid_cfdi IS NULL",
        )
        .bind(c.sueldo_bruto)
        .bind(c.isr)
        .bind(c.imss_obrero)
        .bind(c.infonavit_desc)
        .bind(c.otras_deducciones)
        .bind(c.total_percepciones)
        .bind(c.total_deducciones)
        .bind(c.neto)
        .bind(c.cuota_patronal_imss)
        .bind(c.aportacion_infonavit)
        .bind(c.aportacion_sar)
        .bind(c.isn)
        .bind(c.costo_total_patron)
        .bind(c.horas_extra_dobles)
        .bind(c.horas_extra_triples)
        .bind(c.horas_extra_importe)
        .bind(c.horas_extra_exento)
        .bind(c.horas_extra_gravado)
        .bind(id)
        .execute(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;
        recibos_recalculados += res.rows_affected() as usize;
    }

    let antes = serde_json::json!({
        "periodo_id": periodo_id,
        "fecha_pago": periodo.fecha_pago,
        "estado": periodo.estado,
        "neto_total": neto_antes_total,
    });
    let despues = serde_json::json!({
        "motivo": motivo,
        "periodo_id": periodo_id,
        "fecha_pago_original": periodo.fecha_pago,
        "parametros_vigentes": {
            "uma_diaria": uma_diaria,
            "subsidio_maximo": subsidio_maximo,
            "subsidio_tope": subsidio_tope,
        },
        "recibos_recalculados": recibos_recalculados,
        "diferencias": diferencias.len(),
        "delta_neto_total": delta_neto_total,
        "cfdi_ajuste": "PENDIENTE_V1_1",
    });

    sqlx::query(
        "INSERT INTO audit_log (tabla, operacion, registro_id, antes_json, despues_json, usuario) \
         VALUES (?,?,?,?,?,?)",
    )
    .bind("recibos")
    .bind("RECALCULO")
    .bind(periodo_id)
    .bind(antes.to_string())
    .bind(despues.to_string())
    .bind(&usuario)
    .execute(&mut *tx)
    .await
    .map_err(|e| e.to_string())?;

    tx.commit().await.map_err(|e| e.to_string())?;

    Ok(ResultadoRecalculo {
        recibos_recalculados,
        diferencias,
    })
}

/// Recalcula TODOS los recibos de un periodo (abierto o cerrado) con el motivo
/// obligatorio y los parámetros vigentes a su fecha de pago original.
pub async fn recalcular_periodo_pool(
    p: &SqlitePool,
    periodo_id: i64,
    motivo: &str,
) -> Result<ResultadoRecalculo, String> {
    recalcular_pool(p, periodo_id, motivo, None).await
}

/// Recalcula un solo trabajador dentro de un periodo. La firma pública no
/// incluye motivo, pero se registra uno de sistema no vacío en audit_log.
pub async fn recalcular_trabajador_periodo_pool(
    p: &SqlitePool,
    trabajador_id: i64,
    periodo_id: i64,
) -> Result<ResultadoRecalculo, String> {
    recalcular_pool(p, periodo_id, MOTIVO_INDIVIDUAL, Some(trabajador_id)).await
}

#[tauri::command]
pub async fn recalcular_periodo(
    app: AppHandle,
    periodo_id: i64,
    motivo: String,
) -> Result<ResultadoRecalculo, String> {
    let p = pool(&app).await?;
    recalcular_periodo_pool(&p, periodo_id, &motivo).await
}

#[tauri::command]
pub async fn recalcular_trabajador_periodo(
    app: AppHandle,
    trabajador_id: i64,
    periodo_id: i64,
) -> Result<ResultadoRecalculo, String> {
    let p = pool(&app).await?;
    recalcular_trabajador_periodo_pool(&p, trabajador_id, periodo_id).await
}
