// INFONAVIT (Ley del INFONAVIT, art. 97 LFT).
// Aportacion patronal: 5% del SBC.
// Descuento al trabajador: segun factor/tipo de credito, con tope del 20%
// del salario minimo (art. 97 LFT).
// Reforma DOF feb-2025: el descuento NO se suspende en incapacidad.
// Implementado en calcular_infonavit (linea 15): descuento se calcula
// siempre que activo=true, sin restar dias de incapacidad.

#[derive(Debug, Clone)]
pub struct DescuentoInfonavit {
    pub aportacion_patronal: f64, // 5% SBC
    pub descuento_trabajador: f64,
}

pub fn calcular_infonavit(
    sbc_total: f64,
    salario_diario: f64,
    dias: f64,
    factor: f64,
    tipo_descuento: &str, // "CUOTA_FIJA_VSM", "PORCENTAJE", "MONTO"
    activo: bool,
    uma_diaria: f64,
    salario_minimo: f64,
) -> DescuentoInfonavit {
    if !activo {
        return DescuentoInfonavit { aportacion_patronal: sbc_total * 0.05, descuento_trabajador: 0.0 };
    }
    let aportacion = sbc_total * 0.05;
    // tope descuento: 20% del salario minimo diario * dias (art. 97 LFT)
    let tope_descuento = salario_minimo * 0.20 * dias;
    let descuento = match tipo_descuento {
        "PORCENTAJE" => sbc_total * factor,
        "MONTO" => factor * dias,
        _ => {
            // CUOTA_FIJA_VSM: factor en VSM (UMA aprox) por dia
            factor * uma_diaria * dias
        }
    };
    let descuento_final = descuento.min(tope_descuento);
    let _ = salario_diario;
    DescuentoInfonavit {
        aportacion_patronal: aportacion,
        descuento_trabajador: descuento_final,
    }
}

// Reforma DOF feb-2025: el descuento NO se suspende por incapacidad.
// Los dias de incapacidad se reciben solo para documentar el escenario;
// el descuento se calcula sobre los dias nominales del periodo.
pub fn calcular_infonavit_con_incapacidad(
    sbc_total: f64,
    salario_diario: f64,
    dias_nominales: f64,
    _dias_incapacidad: f64,
    factor: f64,
    tipo_descuento: &str,
    activo: bool,
    uma_diaria: f64,
    salario_minimo: f64,
) -> DescuentoInfonavit {
    calcular_infonavit(sbc_total, salario_diario, dias_nominales, factor, tipo_descuento, activo, uma_diaria, salario_minimo)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tope_20_por_ciento() {
        let d = calcular_infonavit(10000.0, 500.0, 30.0, 1.0, "MONTO", true, 117.31, 278.80);
        // tope = 278.80*0.20*30 = 1672.8
        assert!(d.descuento_trabajador <= 1672.8 + 0.01);
    }

    #[test]
    fn inactivo_solo_aportacion() {
        let d = calcular_infonavit(10000.0, 500.0, 30.0, 1.0, "MONTO", false, 117.31, 278.80);
        assert_eq!(d.descuento_trabajador, 0.0);
        assert!(d.aportacion_patronal > 0.0);
    }
}