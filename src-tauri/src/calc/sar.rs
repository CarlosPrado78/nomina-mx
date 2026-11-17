// SAR (Ley del SAR): aportacion patronal 2% del SBC.

pub fn calcular_sar(sbc_total: f64) -> f64 {
    sbc_total * 0.02
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn sar_2_por_ciento() {
        assert!((calcular_sar(10000.0) - 200.0).abs() < 0.01);
    }
}