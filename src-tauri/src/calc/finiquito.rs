// Finiquito por renuncia/terminacion (art. 50 LFT y siguientes):
// salario pendiente + vacaciones + prima vacacional + aguinaldo proporcional.

pub struct Finiquito {
    pub salario_pendiente: f64,
    pub vacaciones: f64,
    pub prima_vacacional: f64,
    pub aguinaldo_proporcional: f64,
    pub total: f64,
}

pub fn calcular_finiquito(
    salario_diario: f64,
    dias_pendientes: f64,
    dias_vacaciones_pendientes: f64,
    prima_vac_pct: f64,
    dias_trabajados_anio: f64,
    dias_aguinaldo_minimos: f64,
) -> Finiquito {
    let salario_pendiente = salario_diario * dias_pendientes;
    let vacaciones = salario_diario * dias_vacaciones_pendientes;
    let prima_vacacional = vacaciones * prima_vac_pct;
    let aguinaldo_proporcional =
        salario_diario * (dias_aguinaldo_minimos * dias_trabajados_anio / 365.0);
    let total = salario_pendiente + vacaciones + prima_vacacional + aguinaldo_proporcional;
    Finiquito {
        salario_pendiente,
        vacaciones,
        prima_vacacional,
        aguinaldo_proporcional,
        total,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn finiquito_2_anios() {
        let f = calcular_finiquito(278.80, 5.0, 12.0, 0.25, 180.0, 15.0);
        assert!(f.total > 0.0);
    }
}