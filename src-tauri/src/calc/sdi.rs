// Salario Diario Integrado (art. 27 LSS).
// SDI = salario_diario * factor_integracion
// factor = 1 + (dias_aguinaldo + dias_vacaciones*prima_vac +
//          dias_aguinaldo*dias_vacaciones*prima_vac/365)/365
// Tope: 25 UMA diarias.

pub fn calcular_factor_integracion(
    dias_aguinaldo: f64,
    dias_vacaciones: f64,
    prima_vacacional: f64,
) -> f64 {
    let prima = prima_vacacional; // fraccion, ej 0.25
    let numerador = dias_aguinaldo
        + dias_vacaciones * prima
        + (dias_aguinaldo * dias_vacaciones * prima) / 365.0;
    1.0 + numerador / 365.0
}

pub fn calcular_sdi(
    salario_diario: f64,
    dias_aguinaldo: f64,
    dias_vacaciones: f64,
    prima_vacacional: f64,
    uma_diaria: f64,
) -> f64 {
    let factor = calcular_factor_integracion(dias_aguinaldo, dias_vacaciones, prima_vacacional);
    let sdi = salario_diario * factor;
    let tope = 25.0 * uma_diaria;
    if sdi > tope {
        tope
    } else {
        sdi
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn factor_antiguedad_1_anio() {
        // 15 dias aguinaldo, 12 dias vacaciones, prima 25%
        let f = calcular_factor_integracion(15.0, 12.0, 0.25);
        // 1 + (15 + 12*0.25 + 15*12*0.25/365)/365 = 1.04965
        assert!((f - 1.04965).abs() < 0.0001);
    }

    #[test]
    fn tope_25_uma() {
        let uma = 117.31;
        let sdi = calcular_sdi(10000.0, 15.0, 12.0, 0.25, uma);
        assert!((sdi - 25.0 * uma).abs() < 0.01);
    }
}