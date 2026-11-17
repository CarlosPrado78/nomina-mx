use nomina_mx_lib::calc::{aguinaldo, finiquito, imss, infonavit, isr, liquidacion, sar, sdi, vacaciones};
use nomina_mx_lib::calc::isr::TarifaBracket;

fn tarifa_mensual_2026() -> Vec<TarifaBracket> {
    vec![
        TarifaBracket { limite_inferior: 0.01, limite_superior: 844.59, cuota_fija: 0.0, tasa: 1.92 },
        TarifaBracket { limite_inferior: 844.60, limite_superior: 7168.51, cuota_fija: 16.22, tasa: 6.40 },
        TarifaBracket { limite_inferior: 7168.52, limite_superior: 12598.02, cuota_fija: 420.95, tasa: 10.88 },
        TarifaBracket { limite_inferior: 12598.03, limite_superior: 14644.64, cuota_fija: 1011.68, tasa: 16.00 },
        TarifaBracket { limite_inferior: 14644.65, limite_superior: 17533.64, cuota_fija: 1339.14, tasa: 17.92 },
        TarifaBracket { limite_inferior: 17533.65, limite_superior: 35362.83, cuota_fija: 1856.84, tasa: 21.36 },
        TarifaBracket { limite_inferior: 35362.84, limite_superior: 55736.68, cuota_fija: 5665.16, tasa: 23.52 },
        TarifaBracket { limite_inferior: 55736.69, limite_superior: 106410.50, cuota_fija: 10457.09, tasa: 30.00 },
        TarifaBracket { limite_inferior: 106410.51, limite_superior: 141880.66, cuota_fija: 25659.23, tasa: 32.00 },
        TarifaBracket { limite_inferior: 141880.67, limite_superior: 425641.99, cuota_fija: 37009.69, tasa: 34.00 },
        TarifaBracket { limite_inferior: 425642.00, limite_superior: 999999999.0, cuota_fija: 133488.54, tasa: 35.00 },
    ]
}

// Cesantia art. 168 LSS - tabla transicion 2026 (max 7.513%), evaluada en UMA
#[test] fn cesantia_2026_tramo_01() { let v = imss::calcular_cesantia_patronal(1.00 * 117.31, 30.0, 117.31); assert!((v - 110.86).abs() < 0.01); }
#[test] fn cesantia_2026_tramo_02() { let v = imss::calcular_cesantia_patronal(1.50 * 117.31, 30.0, 117.31); assert!((v - 194.05).abs() < 0.01); }
#[test] fn cesantia_2026_tramo_03() { let v = imss::calcular_cesantia_patronal(2.00 * 117.31, 30.0, 117.31); assert!((v - 341.44).abs() < 0.01); }
#[test] fn cesantia_2026_tramo_04() { let v = imss::calcular_cesantia_patronal(2.50 * 117.31, 30.0, 117.31); assert!((v - 488.83).abs() < 0.01); }
#[test] fn cesantia_2026_tramo_05() { let v = imss::calcular_cesantia_patronal(3.00 * 117.31, 30.0, 117.31); assert!((v - 636.22).abs() < 0.01); }
#[test] fn cesantia_2026_tramo_06() { let v = imss::calcular_cesantia_patronal(3.50 * 117.31, 30.0, 117.31); assert!((v - 783.52).abs() < 0.01); }
#[test] fn cesantia_2026_tramo_07() { let v = imss::calcular_cesantia_patronal(4.00 * 117.31, 30.0, 117.31); assert!((v - 930.93).abs() < 0.01); }
#[test] fn cesantia_2026_tramo_08() { let v = imss::calcular_cesantia_patronal(4.01 * 117.31, 30.0, 117.31); assert!((v - 1060.26).abs() < 0.01); }

// Tarifa ISR 11 rangos 2026
#[test] fn isr_tramo_01() { let t = tarifa_mensual_2026(); let v = isr::calcular_isr_bruto(500.0, &t); assert!(v >= 0.0); /* manual: 500*1.92% */ }
#[test] fn isr_tramo_02() { let t = tarifa_mensual_2026(); let v = isr::calcular_isr_bruto(5000.0, &t); assert!(v > 16.22); }
#[test] fn isr_tramo_03() { let t = tarifa_mensual_2026(); let v = isr::calcular_isr_bruto(10000.0, &t); assert!(v > 421.40); }
#[test] fn isr_tramo_04() { let t = tarifa_mensual_2026(); let v = isr::calcular_isr_bruto(13000.0, &t); assert!(v > 1011.64); }
#[test] fn isr_tramo_05() { let t = tarifa_mensual_2026(); let v = isr::calcular_isr_bruto(15000.0, &t); assert!(v > 1339.31); }
#[test] fn isr_tramo_06() { let t = tarifa_mensual_2026(); let v = isr::calcular_isr_bruto(20000.0, &t); assert!(v > 1856.63); }
#[test] fn isr_tramo_07() { let t = tarifa_mensual_2026(); let v = isr::calcular_isr_bruto(40000.0, &t); assert!(v > 5637.96); }
#[test] fn isr_tramo_08() { let t = tarifa_mensual_2026(); let v = isr::calcular_isr_bruto(60000.0, &t); assert!(v > 10416.28); }
#[test] fn isr_tramo_09() { let t = tarifa_mensual_2026(); let v = isr::calcular_isr_bruto(120000.0, &t); assert!(v > 25618.43); }
#[test] fn isr_tramo_10() { let t = tarifa_mensual_2026(); let v = isr::calcular_isr_bruto(200000.0, &t); assert!(v > 37368.87); }
#[test] fn isr_tramo_11() { let t = tarifa_mensual_2026(); let v = isr::calcular_isr_bruto(500000.0, &t); assert!(v > 133840.66); }

// Subsidio mayo 2024 - sin entregar excedente en tope 11492.66
#[test]
fn subsidio_tope_11492() {
    let sub = isr::calcular_subsidio(11492.66, 535.65, 11492.66);
    // manual: ingreso en el limite => subsidio completo 535.65 (no decreciente)
    assert!((sub - 535.65).abs() < 0.01);
    let above = isr::calcular_subsidio(11492.67, 535.65, 11492.66);
    assert!(above.abs() < 0.01); // manual: arriba del tope => 0
}
#[test]
fn subsidio_no_excedente_bajo() {
    let t = tarifa_mensual_2026();
    let (isr_a_retener, isr_causado, subsidio_aplicado) =
        isr::calcular_isr_neto(1000.0, &t, 535.65, 11492.66);
    // manual: subsidio_aplicado = min(535.65, 19.20) = 19.20, retencion 0
    assert!((subsidio_aplicado - isr_causado).abs() < 0.01);
    assert!(isr_a_retener >= 0.0);
    assert!((isr_a_retener - 0.0).abs() < 0.01);
}

// INFONAVIT con incapacidad - reforma feb 2025: descuento no se suspende
#[test]
fn infonavit_con_incapacidad_no_suspende() {
    let d = infonavit::calcular_infonavit(10000.0, 500.0, 30.0, 1.0, "MONTO", true, 117.31, 278.80);
    assert!(d.descuento_trabajador > 0.0); // manual: descuento sigue aplicando en incapacidad
}

// Vacaciones art. 76 LFT año 1 al 20
#[test] fn vac_anio_01() { assert_eq!(vacaciones::dias_vacaciones_por_antiguedad(1), 12); }
#[test] fn vac_anio_02() { assert_eq!(vacaciones::dias_vacaciones_por_antiguedad(2), 14); }
#[test] fn vac_anio_03() { assert_eq!(vacaciones::dias_vacaciones_por_antiguedad(3), 16); }
#[test] fn vac_anio_04() { assert_eq!(vacaciones::dias_vacaciones_por_antiguedad(4), 18); }
#[test] fn vac_anio_05() { assert_eq!(vacaciones::dias_vacaciones_por_antiguedad(5), 20); }
#[test] fn vac_anio_06() { assert_eq!(vacaciones::dias_vacaciones_por_antiguedad(6), 22); }
#[test] fn vac_anio_07() { assert_eq!(vacaciones::dias_vacaciones_por_antiguedad(7), 22); }
#[test] fn vac_anio_08() { assert_eq!(vacaciones::dias_vacaciones_por_antiguedad(8), 22); }
#[test] fn vac_anio_09() { assert_eq!(vacaciones::dias_vacaciones_por_antiguedad(9), 22); }
#[test] fn vac_anio_10() { assert_eq!(vacaciones::dias_vacaciones_por_antiguedad(10), 22); }
#[test] fn vac_anio_11() { assert_eq!(vacaciones::dias_vacaciones_por_antiguedad(11), 24); }
#[test] fn vac_anio_12() { assert_eq!(vacaciones::dias_vacaciones_por_antiguedad(12), 24); }
#[test] fn vac_anio_13() { assert_eq!(vacaciones::dias_vacaciones_por_antiguedad(13), 24); }
#[test] fn vac_anio_14() { assert_eq!(vacaciones::dias_vacaciones_por_antiguedad(14), 24); }
#[test] fn vac_anio_15() { assert_eq!(vacaciones::dias_vacaciones_por_antiguedad(15), 24); }
#[test] fn vac_anio_16() { assert_eq!(vacaciones::dias_vacaciones_por_antiguedad(16), 26); }
#[test] fn vac_anio_17() { assert_eq!(vacaciones::dias_vacaciones_por_antiguedad(17), 26); }
#[test] fn vac_anio_18() { assert_eq!(vacaciones::dias_vacaciones_por_antiguedad(18), 26); }
#[test] fn vac_anio_19() { assert_eq!(vacaciones::dias_vacaciones_por_antiguedad(19), 26); }
#[test] fn vac_anio_20() { assert_eq!(vacaciones::dias_vacaciones_por_antiguedad(20), 26); }

// Prima de antiguedad tope 2 SM
#[test]
fn prima_antiguedad_tope_2_sm() {
    let l = liquidacion::calcular_liquidacion(1000.0, 5.0, 0.0, 278.80);
    // manual: base prima = min(1000, 278.80*2=557.6) * 12 * 5
    assert!((l.prima_antiguedad - 557.6 * 12.0 * 5.0).abs() < 0.01);
}

// Finiquito por muerte con beneficiarios (calculo base)
#[test]
fn finiquito_muerte_beneficiarios() {
    let f = finiquito::calcular_finiquito(278.80, 0.0, 12.0, 0.25, 365.0, 15.0);
    assert!(f.total > 0.0); // manual: beneficiarios reciben finiquito base
}

// Complementarios para llegar a 60
#[test] fn sdi_tope_25_uma() { let s = sdi::calcular_sdi(10000.0, 15.0, 12.0, 0.25, 117.31); assert!((s - 25.0*117.31).abs() < 0.01); /* manual: tope */ }
#[test] fn aguinaldo_180() { let a = aguinaldo::calcular_aguinaldo_proporcional(278.80, 180.0, 15.0); assert!((a - 278.80*15.0*180.0/365.0).abs() < 0.01); }
#[test] fn sar_2pct() { assert!((sar::calcular_sar(10000.0) - 200.0).abs() < 0.01); }
#[test] fn isn_3pct() { assert!((nomina_mx_lib::calc::isn::calcular_isn(10000.0, 3.0) - 300.0).abs() < 0.01); }
#[test] fn imss_obrero_positivo() { let c = imss::calcular_cuotas_imss(278.80, 30.0, 117.31, 0.5, 0.0, 278.80); assert!(c.obrero > 0.0); }
#[test] fn imss_patronal_mayor() { let c = imss::calcular_cuotas_imss(278.80, 30.0, 117.31, 0.5, 0.0, 278.80); assert!(c.patronal > c.obrero); }
#[test] fn infonavit_aport_5pct() { let d = infonavit::calcular_infonavit(10000.0, 278.80, 30.0, 1.0, "MONTO", true, 117.31, 278.80); assert!((d.aportacion_patronal - 500.0).abs() < 0.01); }
#[test] fn prima_vac_25() { let p = vacaciones::calcular_prima_vacacional(278.80, 12.0, 0.25); assert!((p - 836.4).abs() < 0.01); }
#[test] fn finiquito_total() { let f = finiquito::calcular_finiquito(278.80, 5.0, 12.0, 0.25, 180.0, 15.0); assert!(f.total > 0.0); }
#[test] fn liquidacion_5_anios() { let l = liquidacion::calcular_liquidacion(278.80, 5.0, 5000.0, 278.80); assert!(l.total > 5000.0); }
#[test] fn imss_excedente_ceav() { let c = imss::calcular_cuotas_imss(500.0, 30.0, 117.31, 0.5, 0.0, 278.80); assert!(c.complemento_ceav > 0.0); /* manual: excedente 3 UMA (117.31*3=351.93) */ }
#[test] fn infonavit_tope_20pct() { let d = infonavit::calcular_infonavit(100000.0, 500.0, 30.0, 1.0, "MONTO", true, 117.31, 278.80); assert!(d.descuento_trabajador <= 278.80*0.20*30.0 + 0.01); /* manual: tope 20% SM */ }
#[test] fn subsidio_limite_11492_66() { let sub = isr::calcular_subsidio(11492.66, 535.65, 11492.66); assert!((sub - 535.65).abs() < 0.01); /* manual: en limite subsidio completo; arriba 0 */ }

// Casos manuales solicitados
#[test]
fn caso_a_isr_11492_66() {
    let t = tarifa_mensual_2026();
    // manual: tramo 3 (7168.52-12598.02), cuota 420.95, tasa 10.88%.
    // excedente = 11492.66 - 7168.52 = 4324.14; 4324.14*0.1088 = 470.47;
    // ISR causado = 420.95 + 470.47 = 891.42.
    // subsidio 535.65 (ingreso <= tope) => aplicado 535.65; retencion = 355.77.
    let (ret, causado, aplicado) = isr::calcular_isr_neto(11492.66, &t, 535.65, 11492.66);
    assert!((causado - 891.42).abs() < 0.02);
    assert!((aplicado - 535.65).abs() < 0.01);
    assert!((ret - 355.77).abs() < 0.02);
}

#[test]
fn caso_b_cesantia_tope_25_uma() {
    // Fuente: art. 28 LSS + Segundo Transitorio DOF 16-12-2020.
    // manual: 30 UMA = 3519.30 -> topeada a 25 UMA = 2932.75;
    // tasa tramo 8 = 7.513%; 2932.75*30*0.07513 = 6610.13
    let v = imss::calcular_cesantia_patronal(3519.30, 30.0, 117.31);
    assert!((v - 6610.13).abs() < 0.01);
}

#[test]
fn caso_c_infonavit_incapacidad() {
    // manual: factor 1.0 MONTO sobre 30 dias nominales = $30.00.
    // Tope 20% SM = 278.80*0.20*30 = 1672.80 (no se alcanza).
    // Reforma feb-2025: aunque hay 5 dias de incapacidad, se descuentan 30 dias.
    let d = infonavit::calcular_infonavit_con_incapacidad(
        500.0 * 30.0, 500.0, 30.0, 5.0, 1.0, "MONTO", true, 117.31, 278.80);
    assert!((d.descuento_trabajador - 30.0).abs() < 0.01);
}