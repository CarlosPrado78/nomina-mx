// Liquidacion por despido injustificado (art. 50 LFT):
// finiquito + 3 meses de salario + 20 dias por anio + prima de antiguedad
// (12 dias por anio, tope 2 SM).

pub struct Liquidacion {
    pub finiquito: f64,
    pub tres_meses: f64,
    pub veinte_dias_por_anio: f64,
    pub prima_antiguedad: f64,
    pub total: f64,
}

pub fn calcular_liquidacion(
    salario_diario: f64,
    anios_antiguedad: f64,
    finiquito_base: f64,
    salario_minimo: f64,
) -> Liquidacion {
    let tres_meses = salario_diario * 90.0; // 3 meses ~ 90 dias
    let veinte_dias_por_anio = salario_diario * 20.0 * anios_antiguedad;
    // prima de antiguedad: 12 dias por anio, tope 2 SM diarios
    let tope_prima = salario_minimo * 2.0;
    let base_prima = salario_diario.min(tope_prima);
    let prima_antiguedad = base_prima * 12.0 * anios_antiguedad;
    let total = finiquito_base + tres_meses + veinte_dias_por_anio + prima_antiguedad;
    Liquidacion {
        finiquito: finiquito_base,
        tres_meses,
        veinte_dias_por_anio,
        prima_antiguedad,
        total,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn liquidacion_5_anios() {
        let l = calcular_liquidacion(278.80, 5.0, 5000.0, 278.80);
        assert!(l.total > l.finiquito);
        assert!(l.prima_antiguedad > 0.0);
    }
}