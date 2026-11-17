use nomina_mx_lib::calc::imss;

const UMA: f64 = 117.31;

// Fuente: articulo 168 LSS, fraccion II inciso a), y Segundo Transitorio
// del Decreto DOF 16-12-2020 (tabla de gradualidad 2023-2030).
// Cita textual del DOF: "La cuota patronal prevista en el articulo 168,
// fraccion II, inciso a), de la Ley del Seguro Social sera aplicable de
// manera gradual, a partir del 1 de enero de 2023, de conformidad con la
// siguiente tabla". Tasas 2026: 3.150%, 3.676%, 4.851%, 5.556%, 6.026%,
// 6.361%, 6.613%, 7.513%.

#[test]
fn tramo_01() {
    // manual: 1.00 UMA = 117.31; 117.31 * 30 * 0.0315 = 110.86
    let v = imss::calcular_cesantia_patronal(1.00 * UMA, 30.0, UMA);
    assert!((v - 110.86).abs() < 0.01);
}

#[test]
fn tramo_02() {
    // manual: 1.50 UMA = 175.965; 175.965 * 30 * 0.03676 = 194.05
    let v = imss::calcular_cesantia_patronal(1.50 * UMA, 30.0, UMA);
    assert!((v - 194.05).abs() < 0.01);
}

#[test]
fn tramo_03() {
    // manual: 2.00 UMA = 234.62; 234.62 * 30 * 0.04851 = 341.44
    let v = imss::calcular_cesantia_patronal(2.00 * UMA, 30.0, UMA);
    assert!((v - 341.44).abs() < 0.01);
}

#[test]
fn tramo_04() {
    // manual: 2.50 UMA = 293.275; 293.275 * 30 * 0.05556 = 488.83
    let v = imss::calcular_cesantia_patronal(2.50 * UMA, 30.0, UMA);
    assert!((v - 488.83).abs() < 0.01);
}

#[test]
fn tramo_05() {
    // manual: 3.00 UMA = 351.93; 351.93 * 30 * 0.06026 = 636.22
    let v = imss::calcular_cesantia_patronal(3.00 * UMA, 30.0, UMA);
    assert!((v - 636.22).abs() < 0.01);
}

#[test]
fn tramo_06() {
    // manual: 3.50 UMA = 410.585; 410.585 * 30 * 0.06361 = 783.52
    let v = imss::calcular_cesantia_patronal(3.50 * UMA, 30.0, UMA);
    assert!((v - 783.52).abs() < 0.01);
}

#[test]
fn tramo_07() {
    // manual: 4.00 UMA = 469.24; 469.24 * 30 * 0.06613 = 930.93
    let v = imss::calcular_cesantia_patronal(4.00 * UMA, 30.0, UMA);
    assert!((v - 930.93).abs() < 0.01);
}

#[test]
fn tramo_08() {
    // manual: 4.01 UMA = 470.4131; 470.4131 * 30 * 0.07513 = 1060.26
    let v = imss::calcular_cesantia_patronal(4.01 * UMA, 30.0, UMA);
    assert!((v - 1060.26).abs() < 0.01);
}

#[test]
fn frontera_1_00_uma() {
    // manual: 117.31 * 30 * 0.0315 = 110.86
    let v = imss::calcular_cesantia_patronal(1.00 * UMA, 30.0, UMA);
    assert!((v - 110.86).abs() < 0.01);
}

#[test]
fn frontera_1_50_uma() {
    // manual: 175.965 * 30 * 0.03676 = 194.05 (tasa 3.676%)
    let v = imss::calcular_cesantia_patronal(1.50 * UMA, 30.0, UMA);
    assert!((v - 194.05).abs() < 0.01);
}

#[test]
fn frontera_1_51_uma() {
    // manual: 177.1381 * 30 * 0.04851 = 257.79 (tasa 4.851%)
    let v = imss::calcular_cesantia_patronal(1.51 * UMA, 30.0, UMA);
    assert!((v - 257.79).abs() < 0.01);
}

#[test]
fn frontera_2_00_uma() {
    // manual: 234.62 * 30 * 0.04851 = 341.44
    let v = imss::calcular_cesantia_patronal(2.00 * UMA, 30.0, UMA);
    assert!((v - 341.44).abs() < 0.01);
}

#[test]
fn frontera_2_01_uma() {
    // manual: 235.7931 * 30 * 0.05556 = 393.02 (tasa 5.556%)
    let v = imss::calcular_cesantia_patronal(2.01 * UMA, 30.0, UMA);
    assert!((v - 393.02).abs() < 0.01);
}

#[test]
fn frontera_2_50_uma() {
    // manual: 293.275 * 30 * 0.05556 = 488.83
    let v = imss::calcular_cesantia_patronal(2.50 * UMA, 30.0, UMA);
    assert!((v - 488.83).abs() < 0.01);
}

#[test]
fn frontera_3_00_uma() {
    // manual: 351.93 * 30 * 0.06026 = 636.22
    let v = imss::calcular_cesantia_patronal(3.00 * UMA, 30.0, UMA);
    assert!((v - 636.22).abs() < 0.01);
}

#[test]
fn frontera_3_50_uma() {
    // manual: 410.585 * 30 * 0.06361 = 783.52
    let v = imss::calcular_cesantia_patronal(3.50 * UMA, 30.0, UMA);
    assert!((v - 783.52).abs() < 0.01);
}

#[test]
fn frontera_4_00_uma() {
    // manual: 469.24 * 30 * 0.06613 = 930.93
    let v = imss::calcular_cesantia_patronal(4.00 * UMA, 30.0, UMA);
    assert!((v - 930.93).abs() < 0.01);
}

#[test]
fn frontera_4_01_uma() {
    // manual: 470.4131 * 30 * 0.07513 = 1060.26
    let v = imss::calcular_cesantia_patronal(4.01 * UMA, 30.0, UMA);
    assert!((v - 1060.26).abs() < 0.01);
}

#[test]
fn tope_sbc_25_uma() {
    // Fuente: articulo 28 LSS (tope SBC 25 UMA).
    // manual: 30 UMA = 3519.30 -> tope 25 UMA = 2932.75
    // 2932.75 * 30 * 0.07513 = 6610.13 (tasa oficial 2026 = 7.5130%)
    let v = imss::calcular_cesantia_patronal(30.0 * UMA, 30.0, UMA);
    assert!((v - 6610.13).abs() < 0.01);
    assert!(v < 30.0 * UMA * 30.0 * 0.07513);
}

#[test]
fn cesantia_obrera_separada() {
    // Fuente: articulo 168 LSS. manual: 586.55 * 30 * 0.01125 = 197.96
    let v = imss::calcular_cesantia_obrera(586.55, 30.0);
    assert!((v - 197.96).abs() < 0.01);
}

#[test]
fn vejez_patronal_separada() {
    // Fuente: articulo 25 Ley del SAR. manual: 586.55 * 30 * 0.02 = 351.93
    let v = imss::calcular_vejez_patronal(586.55, 30.0);
    assert!((v - 351.93).abs() < 0.01);
}

#[test]
fn complemento_ceav_excedente() {
    // Fuente: articulo 25 LSS reformado 2020.
    // manual: excedente = 586.55 - 3*117.31 = 234.62; 234.62*30*0.004 = 28.15
    let v = imss::calcular_complemento_ceav(586.55, 30.0, UMA);
    assert!((v - 28.15).abs() < 0.01);
}

#[test]
fn complemento_ceav_en_3_uma_exactas() {
    // manual: 3*117.31 = 351.93; excedente = 0 -> 0.00
    let v = imss::calcular_complemento_ceav(3.0 * UMA, 30.0, UMA);
    assert!((v - 0.0).abs() < 0.01);
}

#[test]
fn old_rate_only_in_comments() {
    // C4-a: 0.11875 NO debe aparecer en codigo ejecutable; 11.875 solo en comentarios.
    let src = include_str!("../src/calc/imss.rs");
    assert!(!src.contains("0.11875"));
    for line in src.lines() {
        if line.contains("11.875") {
            assert!(line.trim_start().starts_with("//"), "11.875 fuera de comentario: {}", line);
        }
    }
}

#[test]
fn incapacidad_no_reduce_cuotas_patronales() {
    // Fuente: articulos 15 fracc. I, 17 y 27 LSS.
    // manual: con 5 dias de incapacidad se Siguen cotizando los 30 dias:
    // patronal(0 incap) == patronal(5 incap); tolerancia < 0.01
    let sin = imss::calcular_cuotas_imss(586.55, 30.0, UMA, 0.5, 0.0, 278.80);
    let con = imss::calcular_cuotas_imss(586.55, 30.0, UMA, 0.5, 5.0, 278.80);
    assert!((sin.patronal - con.patronal).abs() < 0.01);
}

#[test]
fn incapacidad_no_reduce_cuotas_obreras() {
    // Fuente: articulos 15 fracc. I, 17 y 27 LSS.
    // manual: cuotas obreras sobre SBC x 30 dias, no sobre 25 dias.
    let sin = imss::calcular_cuotas_imss(586.55, 30.0, UMA, 0.5, 0.0, 278.80);
    let con = imss::calcular_cuotas_imss(586.55, 30.0, UMA, 0.5, 5.0, 278.80);
    assert!((sin.obrero - con.obrero).abs() < 0.01);
    // verificacion indirecta de dias: 30 dias presumibles
    assert!(con.obrero > 586.55 * 25.0 * 0.01125);
}

#[test]
fn incapacidad_no_reduce_dias_cotizados() {
    // Fuente: articulo 15 fracc. I y articulo 17 LSS.
    // manual: todos los conceptos deben ser identicos con 0 y con 5 dias
    // de incapacidad porque los dias cotizados siguen siendo 30.
    let sin = imss::calcular_cuotas_imss(586.55, 30.0, UMA, 0.5, 0.0, 278.80);
    let con = imss::calcular_cuotas_imss(586.55, 30.0, UMA, 0.5, 5.0, 278.80);
    assert!((sin.eym_obrero - con.eym_obrero).abs() < 0.01);
    assert!((sin.iv_obrero - con.iv_obrero).abs() < 0.01);
    assert!((sin.cesantia_patronal - con.cesantia_patronal).abs() < 0.01);
    assert!((sin.cesantia_obrera - con.cesantia_obrera).abs() < 0.01);
    assert!((sin.vejez_patronal - con.vejez_patronal).abs() < 0.01);
    assert!((sin.complemento_ceav - con.complemento_ceav).abs() < 0.01);
    assert!((sin.guarderia - con.guarderia).abs() < 0.01);
    assert!((sin.retiro - con.retiro).abs() < 0.01);
}

#[test]
fn sbc_no_cambia_por_incapacidad() {
    // Fuente: articulo 27 LSS. El SBC se determina por salario y
    // prestaciones, no por asistencia; se usa 586.55 * 30 en ambos casos.
    let sin = imss::calcular_cuotas_imss(586.55, 30.0, UMA, 0.5, 0.0, 278.80);
    let con = imss::calcular_cuotas_imss(586.55, 30.0, UMA, 0.5, 5.0, 278.80);
    // cesantia obrera = 586.55 * 30 * 0.01125 = 197.96
    assert!((con.cesantia_obrera - 197.96).abs() < 0.01);
    assert!((sin.cesantia_obrera - con.cesantia_obrera).abs() < 0.01);
    // vejez patronal = 586.55 * 30 * 0.02 = 351.93
    assert!((con.vejez_patronal - 351.93).abs() < 0.01);
}

#[test]
fn cinco_conceptos_patronales_no_son_tasa_combinada() {
    // C4-b: la suma de conceptos patronales es > 0 y siempre menor que
    // SBC * dias * 0.30 (nunca se acerca a una tasa combinada vieja).
    let c = imss::calcular_cuotas_imss(586.55, 30.0, UMA, 0.5, 0.0, 278.80);
    let suma = c.cesantia_patronal
        + c.vejez_patronal
        + c.retiro
        + c.cuota_fija_patronal
        + c.excedente_eym_patronal
        + c.iv_patronal
        + c.guarderia
        + c.rt;
    assert!(suma > 0.0);
    assert!(suma < 586.55 * 30.0 * 0.30);
}
