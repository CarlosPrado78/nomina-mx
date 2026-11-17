use crate::calc::{finiquito as fin, liquidacion as liq, vacaciones};
use serde::{Deserialize, Serialize};
#[derive(Debug, Serialize, Deserialize)]
pub struct FiniquitoResult {
    pub salario_pendiente: f64,
    pub vacaciones: f64,
    pub prima_vacacional: f64,
    pub aguinaldo_proporcional: f64,
    pub total: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LiquidacionResult {
    pub finiquito: f64,
    pub tres_meses: f64,
    pub veinte_dias_por_anio: f64,
    pub prima_antiguedad: f64,
    pub total: f64,
}

#[tauri::command]
pub fn calcular_finiquito(
    salario_diario: f64,
    dias_pendientes: f64,
    dias_vacaciones_pendientes: f64,
    prima_vac_pct: f64,
    dias_trabajados_anio: f64,
    dias_aguinaldo_minimos: f64,
) -> FiniquitoResult {
    let f = fin::calcular_finiquito(salario_diario, dias_pendientes, dias_vacaciones_pendientes, prima_vac_pct, dias_trabajados_anio, dias_aguinaldo_minimos);
    FiniquitoResult {
        salario_pendiente: f.salario_pendiente,
        vacaciones: f.vacaciones,
        prima_vacacional: f.prima_vacacional,
        aguinaldo_proporcional: f.aguinaldo_proporcional,
        total: f.total,
    }
}

#[tauri::command]
pub fn calcular_liquidacion(
    salario_diario: f64,
    anios_antiguedad: f64,
    finiquito_base: f64,
    salario_minimo: f64,
) -> LiquidacionResult {
    let l = liq::calcular_liquidacion(salario_diario, anios_antiguedad, finiquito_base, salario_minimo);
    LiquidacionResult {
        finiquito: l.finiquito,
        tres_meses: l.tres_meses,
        veinte_dias_por_anio: l.veinte_dias_por_anio,
        prima_antiguedad: l.prima_antiguedad,
        total: l.total,
    }
}

#[tauri::command]
pub fn dias_vacaciones(anios: u32) -> u32 {
    vacaciones::dias_vacaciones_por_antiguedad(anios)
}