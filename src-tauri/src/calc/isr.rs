// Retencion ISR (art. 96 LISR). Tarifa del Anexo 8 RMF.
// Regla DOF 01-05-2024: el subsidio para el empleo solo reduce el ISR
// retenido; si el subsidio excede al ISR, no se entrega el excedente.
// Implementado en calcular_isr_neto (linea 39): subsidio_aplicado = min(subsidio_calculado, isr_causado), isr_a_retener = isr_causado - subsidio_aplicado.

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TarifaBracket {
    pub limite_inferior: f64,
    pub limite_superior: f64,
    pub cuota_fija: f64,
    pub tasa: f64, // porcentaje
}

pub fn calcular_isr_bruto(ingreso_gravable: f64, tarifa: &[TarifaBracket]) -> f64 {
    // busca el tramo correspondiente
    let mut bracket = &tarifa[0];
    for b in tarifa {
        if ingreso_gravable >= b.limite_inferior && ingreso_gravable <= b.limite_superior {
            bracket = b;
            break;
        }
        if ingreso_gravable > b.limite_superior {
            bracket = b;
        }
    }
    let excedente = (ingreso_gravable - bracket.limite_inferior).max(0.0);
    bracket.cuota_fija + excedente * (bracket.tasa / 100.0)
}

pub fn calcular_subsidio(ingreso_gravable: f64, subsidio_maximo: f64, tope_subsidio: f64) -> f64 {
    // Decreto DOF 31-12-2025 / regla DOF 01-05-2024: cuota fija mensual
    // completa si el ingreso no rebasa el tope; cero si lo rebasa.
    if ingreso_gravable <= 0.0 || ingreso_gravable > tope_subsidio {
        return 0.0;
    }
    subsidio_maximo
}

pub fn calcular_isr_neto(
    ingreso_gravable: f64,
    tarifa: &[TarifaBracket],
    subsidio_maximo: f64,
    tope_subsidio: f64,
) -> (f64, f64, f64) {
    let isr_causado = calcular_isr_bruto(ingreso_gravable, tarifa);
    let subsidio_calculado = calcular_subsidio(ingreso_gravable, subsidio_maximo, tope_subsidio);
    // Regla DOF 01-05-2024: subsidio_aplicado = min(subsidio_calculado, isr_causado)
    let subsidio_aplicado = subsidio_calculado.min(isr_causado);
    // isr_a_retener = isr_causado - subsidio_aplicado; nunca negativo
    let isr_a_retener = isr_causado - subsidio_aplicado;
    (isr_a_retener, isr_causado, subsidio_aplicado)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tarifa_mensual_sample() -> Vec<TarifaBracket> {
        vec![
            TarifaBracket { limite_inferior: 0.01, limite_superior: 844.59, cuota_fija: 0.0, tasa: 1.92 },
            TarifaBracket { limite_inferior: 844.60, limite_superior: 7168.51, cuota_fija: 16.22, tasa: 6.40 },
            TarifaBracket { limite_inferior: 7168.52, limite_superior: 12598.02, cuota_fija: 420.95, tasa: 10.88 },
        ]
    }

    #[test]
    fn isr_bruto_primer_tramo() {
        let t = tarifa_mensual_sample();
        let isr = calcular_isr_bruto(500.0, &t);
        assert!((isr - (500.0 - 0.01) * 0.0192).abs() < 0.01);
    }

    #[test]
    fn subsidio_no_excedente() {
        let (isr_a_retener, isr_causado, subsidio_aplicado) =
            calcular_isr_neto(1000.0, &tarifa_mensual_sample(), 535.65, 11492.66);
        // subsidio_aplicado = min(535.65, isr_causado) = isr_causado; retencion = 0
        assert!((subsidio_aplicado - isr_causado).abs() < 0.01);
        assert!((isr_a_retener - 0.0).abs() < 0.01);
        assert!(isr_a_retener >= 0.0);
    }
}