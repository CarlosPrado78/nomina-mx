// ISN (Impuesto Sobre Nomina) estatal: tasa estatal sobre base gravable
// de percepciones de nomina (revisar exenciones estatales).

pub fn calcular_isn(base_gravable: f64, tasa_pct: f64) -> f64 {
    base_gravable * (tasa_pct / 100.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn isn_3_por_ciento() {
        assert!((calcular_isn(10000.0, 3.0) - 300.0).abs() < 0.01);
    }
}