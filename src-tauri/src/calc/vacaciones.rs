// Vacaciones (art. 76 LFT, reforma 2023) + prima vacacional 25% (art. 80).

pub fn dias_vacaciones_por_antiguedad(anios: u32) -> u32 {
    // Tabla art. 76 LFT (reforma 2023): 12 dias primer anio, incrementa.
    match anios {
        0 => 0,
        1 => 12,
        2 => 14,
        3 => 16,
        4 => 18,
        5 => 20,
        6..=10 => 22,
        11..=15 => 24,
        16..=20 => 26,
        21..=25 => 28,
        26..=30 => 30,
        _ => 32,
    }
}

pub fn calcular_prima_vacacional(salario_diario: f64, dias_vacaciones: f64, prima_pct: f64) -> f64 {
    salario_diario * dias_vacaciones * prima_pct
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn vacaciones_3_anios() {
        assert_eq!(dias_vacaciones_por_antiguedad(3), 16);
    }
    #[test]
    fn prima_25() {
        let p = calcular_prima_vacacional(278.80, 12.0, 0.25);
        assert!((p - 836.4).abs() < 0.01);
    }
}