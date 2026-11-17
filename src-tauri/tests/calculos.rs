use nomina_mx_lib::calc::{aguinaldo, finiquito, imss, infonavit, isn, liquidacion, sar, sdi, vacaciones};

#[test]
fn caso_1_salario_minimo_mensual() {
    let uma = 113.14;
    let sm = 278.80;
    let salario_diario = 278.80;
    let dias = 30.0;
    let sbc = sdi::calcular_sdi(salario_diario, 15.0, 12.0, 0.25, uma);
    let cuotas = imss::calcular_cuotas_imss(sbc, dias, uma, 0.5, 0.0, 278.80);
    assert!(cuotas.obrero > 0.0);
    assert!(cuotas.patronal > cuotas.obrero);
    let sarv = sar::calcular_sar(sbc * dias);
    assert!((sarv - sbc * dias * 0.02).abs() < 0.01);
    let _ = sm;
}

#[test]
fn caso_2_15000_mensual_sin_infonavit() {
    let uma = 113.14;
    let salario_diario = 15000.0 / 30.0;
    let dias = 30.0;
    let sbc = sdi::calcular_sdi(salario_diario, 15.0, 12.0, 0.25, uma);
    let info = infonavit::calcular_infonavit(sbc * dias, salario_diario, dias, 1.0, "MONTO", false, uma, 278.80);
    assert_eq!(info.descuento_trabajador, 0.0);
    assert!(info.aportacion_patronal > 0.0);
}

#[test]
fn caso_3_15000_con_infonavit_factor_1() {
    let uma = 113.14;
    let salario_diario = 15000.0 / 30.0;
    let dias = 30.0;
    let sbc = sdi::calcular_sdi(salario_diario, 15.0, 12.0, 0.25, uma);
    let info = infonavit::calcular_infonavit(sbc * dias, salario_diario, dias, 1.0, "MONTO", true, uma, 278.80);
    assert!(info.descuento_trabajador > 0.0);
}

#[test]
fn caso_4_superior_3_uma_excedente() {
    let uma = 113.14;
    let salario_diario = 500.0; // > 3 uma
    let cuotas = imss::calcular_cuotas_imss(salario_diario, 30.0, uma, 0.5, 0.0, 278.80);
    // excedente CEAV aplicado
    assert!(cuotas.complemento_ceav > 0.0);
}

#[test]
fn caso_5_finiquito_2_anios() {
    let f = finiquito::calcular_finiquito(278.80, 5.0, 12.0, 0.25, 180.0, 15.0);
    assert!(f.total > 0.0);
    assert!((f.prima_vacacional - f.vacaciones * 0.25).abs() < 0.01);
}

#[test]
fn caso_6_liquidacion_5_anios() {
    let l = liquidacion::calcular_liquidacion(278.80, 5.0, 5000.0, 278.80);
    assert!(l.total > 5000.0);
    assert!((l.tres_meses - 278.80 * 90.0).abs() < 0.01);
}

#[test]
fn caso_7_aguinaldo_180_dias() {
    let a = aguinaldo::calcular_aguinaldo_proporcional(278.80, 180.0, 15.0);
    assert!((a - 278.80 * 15.0 * 180.0 / 365.0).abs() < 0.01);
}

#[test]
fn caso_8_vacaciones_3_anios() {
    assert_eq!(vacaciones::dias_vacaciones_por_antiguedad(3), 16);
    let prima = vacaciones::calcular_prima_vacacional(278.80, 16.0, 0.25);
    assert!((prima - 278.80 * 16.0 * 0.25).abs() < 0.01);
}

#[test]
fn isn_aplicacion() {
    assert!((isn::calcular_isn(10000.0, 3.0) - 300.0).abs() < 0.01);
}