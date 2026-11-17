// Cuotas obrero-patronales IMSS (LSS).
// Refactor Fase 9 / A2: los conceptos de CEAV se calculan SEPARADAMENTE.
// La tasa 11.875% es el objetivo 2030 del art. 168 LSS (Segundo
// Transitorio del Decreto DOF 16-12-2020), NO la tasa vigente en 2026.
// En 2026 aplica la tabla de transicion con tasa maxima 7.513%.

#[derive(Debug, Clone, serde::Serialize)]
pub struct CuotasImss {
    pub obrero: f64,
    pub patronal: f64,
    pub cesantia_patronal: f64,
    pub cesantia_obrera: f64,
    pub vejez_patronal: f64,
    pub complemento_ceav: f64,
    pub eym_obrero: f64,
    pub iv_obrero: f64,
    pub cuota_fija_patronal: f64,
    pub excedente_eym_patronal: f64,
    pub iv_patronal: f64,
    pub guarderia: f64,
    pub retiro: f64,
    pub rt: f64,
}

// Articulo 168 LSS, fraccion II, inciso a) y Segundo Transitorio del
// Decreto publicado en el DOF el 16 de diciembre de 2020.
// Cita textual del DOF (Segundo transitorio):
// "La cuota patronal prevista en el articulo 168, fraccion II, inciso a),
//  de la Ley del Seguro Social sera aplicable de manera gradual, a partir
//  del 1 de enero de 2023, de conformidad con la siguiente tabla:"
// La tabla publicada dice literalmente:
//   1.00 SM*                -> 3.150%
//   1.01 SM a 1.50 UMA**    -> 4.202% (objetivo 2030; en 2026: 3.676%)
//   1.51 a 2.00 UMA         -> 6.552% (objetivo 2030; en 2026: 4.851%)
//   2.01 a 2.50 UMA         -> 7.962% (objetivo 2030; en 2026: 5.556%)
//   2.51 a 3.00 UMA         -> 8.902% (objetivo 2030; en 2026: 6.026%)
//   3.01 a 3.50 UMA         -> 9.573% (objetivo 2030; en 2026: 6.361%)
//   3.51 a 4.00 UMA         -> 10.077% (objetivo 2030; en 2026: 6.613%)
//   4.01 UMA en adelante    -> 11.875% (objetivo 2030; en 2026: 7.513%)
//   *Salario Minimo **Unidad de Medida y Actualizacion
// Confirmacion C1: el tramo 1 se mide en SM; el tramo 2 mezcla SM (limite
// inferior) y UMA (limite superior); los tramos 3 a 8 se miden en UMA.
// Como la UMA 2026 ($117.31) es menor que el SM general ($278.80), la
// redaccion literal produce intervalos vacios/traslapados. Para una
// aplicacion determinista se adopta la convencion operativa acordada en
// C2: los 8 tramos se evaluan por veces UMA. Esta convencion queda
// documentada; la tabla oficial se conserva arriba de forma literal.
pub fn tasa_cesantia_patronal_2026(sbc_diario: f64, uma_diaria: f64) -> f64 {
    // Se normaliza a 9 decimales para evitar que 3.50 se evalue como 3.5000000004.
    let veces_uma = if uma_diaria > 0.0 {
        ((sbc_diario / uma_diaria) * 1e9).round() / 1e9
    } else {
        0.0
    };
    if veces_uma <= 1.00 { return 0.0315; }
    if veces_uma <= 1.50 { return 0.03676; }
    if veces_uma <= 2.00 { return 0.04851; }
    if veces_uma <= 2.50 { return 0.05556; }
    if veces_uma <= 3.00 { return 0.06026; }
    if veces_uma <= 3.50 { return 0.06361; }
    if veces_uma <= 4.00 { return 0.06613; }
    0.07513
}

// Articulo 168 LSS + articulo 28 LSS (tope SBC 25 UMA).
pub fn calcular_cesantia_patronal(
    sbc_diario: f64,
    dias: f64,
    uma_diaria: f64,
) -> f64 {
    let sbc_topeado = sbc_diario.min(25.0 * uma_diaria);
    let tasa = tasa_cesantia_patronal_2026(sbc_topeado, uma_diaria);
    ((sbc_topeado * dias * tasa) * 100.0).round() / 100.0
}

// Articulo 168 LSS: cuota obrera de cesantia 1.125% sobre SBC.
pub fn calcular_cesantia_obrera(sbc_diario: f64, dias: f64) -> f64 {
    ((sbc_diario * dias * 0.01125) * 100.0).round() / 100.0
}

// Articulo 25 de la Ley del SAR: aportacion patronal de vejez 2.00%.
pub fn calcular_vejez_patronal(sbc_diario: f64, dias: f64) -> f64 {
    ((sbc_diario * dias * 0.0200) * 100.0).round() / 100.0
}

// Articulo 25 LSS reformado 2020: complemento CEAV 0.40% sobre el
// excedente de 3 UMA diarias (cuota obrera). No se suma a la tasa
// de cesantia patronal.
pub fn calcular_complemento_ceav(sbc_diario: f64, dias: f64, uma_diaria: f64) -> f64 {
    let excedente = (sbc_diario - 3.0 * uma_diaria).max(0.0);
    ((excedente * dias * 0.0040) * 100.0).round() / 100.0
}

// Wrapper de conveniencia: devuelve TODOS los conceptos por separado.
// Nunca devuelve una tasa combinada de 11.875%.
//
// Correccion A2.bis -- fuente legal: articulo 15, fraccion I, articulo 17
// y articulo 27 de la Ley del Seguro Social. La incapacidad no reduce
// dias de cotizacion ni modifica el SBC; las cuotas se causan sobre
// SBC x dias del periodo. El parametro _dias_incapacidad se conserva
// solo para modelar prestaciones economicas en otro modulo; NO se usa
// para descontar dias base. Articulo 304 LSS sanciona la subestimacion.
pub fn calcular_cuotas_imss(
    sbc_diario: f64,
    dias_cotizados: f64,
    uma_diaria: f64,
    prima_rt_pct: f64,
    _dias_incapacidad: f64,
    salario_minimo_diario: f64,
) -> CuotasImss {
    let dias = dias_cotizados;
    let sbc_total = sbc_diario * dias;

    let _ = salario_minimo_diario; // parametro conservado por compatibilidad; no se usa en 2026
    let cesantia_patronal =
        calcular_cesantia_patronal(sbc_diario, dias, uma_diaria);
    let cesantia_obrera = calcular_cesantia_obrera(sbc_diario, dias);
    let vejez_patronal = calcular_vejez_patronal(sbc_diario, dias);
    let complemento_ceav = calcular_complemento_ceav(sbc_diario, dias, uma_diaria);

    // EyM obrero 0.625% (art. 25 LSS), IV obrero 0.625% (art. 147 LSS).
    let eym_obrero = ((sbc_total * 0.00625) * 100.0).round() / 100.0;
    let iv_obrero = ((sbc_total * 0.00625) * 100.0).round() / 100.0;

    // Patronal: cuota fija 20.40% UMA (art. 29 LSS), excedentes 1.10%,
    // IV 1.75%, guarderia 1.00%, retiro 2.00% (art. 29 LSS), RT segun prima.
    let cuota_fija_patronal = ((uma_diaria * 0.2040 * dias) * 100.0).round() / 100.0;
    let excedente_base = (sbc_diario - 3.0 * uma_diaria).max(0.0) * dias;
    let excedente_eym_patronal = ((excedente_base * 0.0110) * 100.0).round() / 100.0;
    let iv_patronal = ((sbc_total * 0.0175) * 100.0).round() / 100.0;
    let guarderia = ((sbc_total * 0.0100) * 100.0).round() / 100.0;
    let retiro = ((sbc_total * 0.0200) * 100.0).round() / 100.0;
    let rt = ((sbc_total * (prima_rt_pct / 100.0)) * 100.0).round() / 100.0;

    let obrero = ((eym_obrero + iv_obrero + cesantia_obrera + complemento_ceav) * 100.0).round() / 100.0;
    let patronal = ((cuota_fija_patronal
        + excedente_eym_patronal
        + iv_patronal
        + guarderia
        + retiro
        + rt
        + cesantia_patronal
        + vejez_patronal)
        * 100.0)
        .round()
        / 100.0;

    CuotasImss {
        obrero,
        patronal,
        cesantia_patronal,
        cesantia_obrera,
        vejez_patronal,
        complemento_ceav,
        eym_obrero,
        iv_obrero,
        cuota_fija_patronal,
        excedente_eym_patronal,
        iv_patronal,
        guarderia,
        retiro,
        rt,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cesantia_tramo_1() {
        // Fuente: art. 168 LSS / Segundo Transitorio DOF 16-12-2020.
        // manual: 117.31 * 30 * 0.0315 = 110.86
        let v = calcular_cesantia_patronal(117.31, 30.0, 117.31);
        assert!((v - 110.86).abs() < 0.01);
    }

    #[test]
    fn conceptos_separados() {
        // Fuente: art. 25 Ley SAR (2%), art. 168 LSS (1.125%), art. 25 LSS ref 2020 (0.40%).
        // manual: vejez 586.55*30*0.02 = 351.93
        assert!((calcular_vejez_patronal(586.55, 30.0) - 351.93).abs() < 0.01);
        // manual: cesantia obrera 586.55*30*0.01125 = 197.96
        assert!((calcular_cesantia_obrera(586.55, 30.0) - 197.96).abs() < 0.01);
        // manual: excedente (586.55 - 351.93)=234.62 *30*0.004 = 28.15
        assert!((calcular_complemento_ceav(586.55, 30.0, 117.31) - 28.15).abs() < 0.01);
    }
}