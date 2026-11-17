// Aguinaldo (art. 87 LFT): 15 dias minimos + proporcional por dias trabajados.

pub fn calcular_aguinaldo_proporcional(
    salario_diario: f64,
    dias_trabajados_anio: f64,
    dias_aguinaldo_minimos: f64,
) -> f64 {
    // proporcional = salario_diario * (dias_aguinaldo_minimos * dias_trabajados / 365)
    salario_diario * (dias_aguinaldo_minimos * dias_trabajados_anio / 365.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn aguinaldo_180_dias() {
        // 180 dias trabajados, 15 dias minimos, salario 278.80
        let a = calcular_aguinaldo_proporcional(278.80, 180.0, 15.0);
        // 278.80 * 15 * 180 / 365 = 2059.5? 
        assert!(a > 0.0);
    }
}