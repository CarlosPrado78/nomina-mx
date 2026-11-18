// Bloque B — Tiempo extraordinario.
//
// Fuentes legales verificadas:
// - LFT art. 66 (texto vigente desde 01-05-2026, DOF 01-05-2026): el tiempo
//   extraordinario "se abonara ... un cien por ciento mas" => 2x del salario
//   ordinario (hora doble). Maximo 12 h/semana, hasta 4 h diarias en 4 dias.
// - Cuarto Transitorio del mismo decreto: en 2026 el tope de horas al doble
//   sigue siendo 9 por semana (va a 12 en 2030).
// - LFT art. 68 (DOF 01-05-2026): la prolongacion que supere el art. 66 no
//   podra exceder 4 h semanales y se paga "un doscientos por ciento mas"
//   => 3x del salario ordinario (hora triple). El pago no legaliza el exceso.
// - LFT art. 61: jornada diurna 8 h, nocturna 7 h, mixta 7.5 h. De ahi se
//   obtiene el valor de la hora ordinaria (salario_diario / horas_jornada).
// - LISR art. 93 fracc. I: para quien percibe el salario minimo, exencion del
//   100% de las remuneraciones por tiempo extraordinario dentro del limite
//   laboral; para los demas, 50%, sin exceder 5 veces la unidad de referencia
//   por semana de servicios. Se usa UMA por la desindexacion del salario
//   minimo (art. 26 apartado C de la CPEUM y transitorios de la reforma 2016).
// - LSS art. 27: las remuneraciones variables por horas extra integran el SBC
//   del periodo; art. 28 LSS: tope de 25 UMA diarias.
//
// Todos los factores, topes y porcentajes vienen de `tablas_fiscales`
// (migracion 0003). No hay constantes fiscales hardcodeadas en esta logica.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParametrosHorasExtra {
    pub factor_dobles: f64,
    pub factor_triples: f64,
    pub limite_dobles_horas: f64,
    pub limite_triples_horas: f64,
    pub porcentaje_exento: f64,
    pub tope_exento_umas_semana: f64,
    pub horas_jornada_diurna: f64,
    pub horas_jornada_nocturna: f64,
    pub horas_jornada_mixta: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DesgloseHorasExtra {
    pub horas_dobles: f64,
    pub horas_triples: f64,
    pub importe_dobles: f64,
    pub importe_triples: f64,
    pub importe_total: f64,
    /// Horas que superan el limite legal (9 + 4 en 2026). Se pagan, pero la
    /// prolongacion sigue siendo infractora (art. 68 LFT, sin perjuicio de sanciones).
    pub horas_excedentes: f64,
    pub excede_limite_legal: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExencionHorasExtra {
    pub exento: f64,
    pub gravado: f64,
    pub base_exenta: f64,
    pub tope_aplicado: f64,
}

fn round2(v: f64) -> f64 {
    (v * 100.0).round() / 100.0
}

/// Horas de la jornada diaria segun tipo (LFT art. 61).
pub fn horas_jornada_diaria(tipo: Option<&str>, p: &ParametrosHorasExtra) -> Result<f64, String> {
    match tipo.unwrap_or("diurna").to_ascii_lowercase().as_str() {
        "diurna" => Ok(p.horas_jornada_diurna),
        "nocturna" => Ok(p.horas_jornada_nocturna),
        "mixta" => Ok(p.horas_jornada_mixta),
        otra => Err(format!("jornada no valida: {} (use diurna, nocturna o mixta)", otra)),
    }
}

/// Valor de una hora ordinaria. Ejemplo: salario diario 400, jornada 8 => 50.
pub fn valor_hora_ordinaria(salario_diario: f64, horas_jornada: f64) -> Result<f64, String> {
    if horas_jornada <= 0.0 {
        return Err("horas de jornada deben ser mayores a cero".to_string());
    }
    if salario_diario < 0.0 {
        return Err("salario_diario no puede ser negativo".to_string());
    }
    Ok(salario_diario / horas_jornada)
}

/// Desglosa horas extraordinarias en dobles y triples.
///
/// Caso canonico del checklist (2026): 12 horas => 9 dobles + 3 triples.
/// Manual: valor hora 50; 9 * 50 * 2 = 900; 3 * 50 * 3 = 450; total 1350.
pub fn desglosar_horas_extra(
    horas: f64,
    salario_diario: f64,
    horas_jornada: f64,
    p: &ParametrosHorasExtra,
) -> Result<DesgloseHorasExtra, String> {
    if horas < 0.0 {
        return Err("horas extraordinarias no pueden ser negativas".to_string());
    }
    let valor_hora = valor_hora_ordinaria(salario_diario, horas_jornada)?;

    let horas_dobles = horas.min(p.limite_dobles_horas);
    let restantes = (horas - horas_dobles).max(0.0);
    // Las horas que exceden el tope de dobles se pagan al triple. Las que
    // ademas exceden el limite de 4 horas del art. 68 se pagan igual, pero se
    // marcan como excedentes: pagarlas no subsana la infraccion.
    let horas_triples = restantes;
    let horas_excedentes = (restantes - p.limite_triples_horas).max(0.0);

    let importe_dobles = round2(horas_dobles * valor_hora * p.factor_dobles);
    let importe_triples = round2(horas_triples * valor_hora * p.factor_triples);
    let importe_total = round2(importe_dobles + importe_triples);

    Ok(DesgloseHorasExtra {
        horas_dobles,
        horas_triples,
        importe_dobles,
        importe_triples,
        importe_total,
        horas_excedentes,
        excede_limite_legal: horas_excedentes > 0.0,
    })
}

/// Exencion ISR del art. 93 fracc. I LISR.
///
/// Solo es exenta la parte pagada dentro del limite laboral (horas dobles).
/// La parte de horas triples se considera remuneracion que excede el limite
/// y queda integramente gravada, criterio consistente con la guia SAT y con
/// la practica de calculo del tope semanal.
///
/// Manual (trabajador no minimo, UMA 117.31, una semana):
///   importe_dobles 900 * 50% = 450; tope 5 * 117.31 = 586.55; exento 450.
pub fn calcular_exencion(
    importe_dobles: f64,
    importe_triples: f64,
    es_salario_minimo: bool,
    uma_diaria: f64,
    semanas: f64,
    p: &ParametrosHorasExtra,
) -> ExencionHorasExtra {
    let base_exenta = if es_salario_minimo {
        importe_dobles
    } else {
        importe_dobles * p.porcentaje_exento
    };
    let tope = if es_salario_minimo {
        f64::INFINITY
    } else {
        p.tope_exento_umas_semana * uma_diaria * semanas.max(1.0)
    };
    let exento = round2(base_exenta.min(tope));
    let gravado = round2(importe_dobles + importe_triples - exento);
    ExencionHorasExtra {
        exento,
        gravado,
        base_exenta: round2(base_exenta),
        tope_aplicado: if tope.is_finite() { round2(tope) } else { 0.0 },
    }
}

/// SBC del periodo: el pago de horas extra es salario variable que se
/// prorratea entre los dias trabajados (LSS art. 27); se aplica el tope
/// de 25 UMA (LSS art. 28).
pub fn integrar_en_sbc(
    sbc_base_diario: f64,
    importe_total_horas_extra: f64,
    dias: f64,
    uma_diaria: f64,
) -> Result<f64, String> {
    if dias <= 0.0 {
        return Err("dias del periodo deben ser mayores a cero".to_string());
    }
    let sbc = sbc_base_diario + importe_total_horas_extra / dias;
    Ok(sbc.min(25.0 * uma_diaria))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn params() -> ParametrosHorasExtra {
        // Copia de la fila vigente 2026 de migrations/0003 (no se usa en produccion).
        ParametrosHorasExtra {
            factor_dobles: 2.0,
            factor_triples: 3.0,
            limite_dobles_horas: 9.0,
            limite_triples_horas: 4.0,
            porcentaje_exento: 0.5,
            tope_exento_umas_semana: 5.0,
            horas_jornada_diurna: 8.0,
            horas_jornada_nocturna: 7.0,
            horas_jornada_mixta: 7.5,
        }
    }

    #[test]
    fn valor_hora_diurna() {
        // Fuente: LFT art. 61 (jornada diurna de 8 horas).
        assert_eq!(valor_hora_ordinaria(400.0, 8.0).unwrap(), 50.0);
    }

    #[test]
    fn doce_horas_nueve_dobles_tres_triples() {
        // Fuente: LFT art. 66 + Cuarto Transitorio DOF 01-05-2026 (9 h en 2026),
        // art. 68 (200% adicional para el excedente).
        // Manual: 9*50*2=900; 3*50*3=450; total=1350.
        let d = desglosar_horas_extra(12.0, 400.0, 8.0, &params()).unwrap();
        assert_eq!(d.horas_dobles, 9.0);
        assert_eq!(d.horas_triples, 3.0);
        assert_eq!(d.importe_dobles, 900.0);
        assert_eq!(d.importe_triples, 450.0);
        assert_eq!(d.importe_total, 1350.0);
        assert!(!d.excede_limite_legal);
    }

    #[test]
    fn trece_horas_es_limite_y_catorce_excede() {
        let p = params();
        let d13 = desglosar_horas_extra(13.0, 400.0, 8.0, &p).unwrap();
        assert_eq!(d13.horas_triples, 4.0);
        assert!(!d13.excede_limite_legal);
        let d14 = desglosar_horas_extra(14.0, 400.0, 8.0, &p).unwrap();
        assert_eq!(d14.horas_triples, 5.0);
        assert_eq!(d14.horas_excedentes, 1.0);
        assert!(d14.excede_limite_legal);
    }

    #[test]
    fn exencion_50_pct_dentro_del_tope() {
        // Fuente: LISR art. 93 fracc. I. Manual: 900*0.5=450 < 5*117.31=586.55.
        let e = calcular_exencion(900.0, 450.0, false, 117.31, 1.0, &params());
        assert_eq!(e.exento, 450.0);
        assert_eq!(e.gravado, 900.0);
        assert_eq!(e.tope_aplicado, 586.55);
    }

    #[test]
    fn exencion_topada_por_cinco_umas() {
        // Manual: 1800*0.5=900 > 5*117.31=586.55; total 1800+900=2700;
        // gravado = 2700-586.55 = 2113.45.
        let e = calcular_exencion(1800.0, 900.0, false, 117.31, 1.0, &params());
        assert_eq!(e.exento, 586.55);
        assert_eq!(e.gravado, 2113.45);
    }

    #[test]
    fn salario_minimo_exencion_total_de_dobles() {
        // Fuente: LISR art. 93 fracc. I, primer supuesto.
        let e = calcular_exencion(900.0, 450.0, true, 117.31, 1.0, &params());
        assert_eq!(e.exento, 900.0);
        assert_eq!(e.gravado, 450.0);
    }

    #[test]
    fn integra_sbc_con_tope_25_uma() {
        // Fuente: LSS arts. 27 y 28. Manual: 400 + 1350/30 = 445; tope 25*117.31=2932.75.
        let sbc = integrar_en_sbc(400.0, 1350.0, 30.0, 117.31).unwrap();
        assert_eq!(sbc, 445.0);
        let topado = integrar_en_sbc(4000.0, 1350.0, 30.0, 117.31).unwrap();
        assert_eq!(topado, 2932.75);
    }
}
