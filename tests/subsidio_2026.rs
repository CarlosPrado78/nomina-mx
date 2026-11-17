// Verificacion del Decreto DOF 31-12-2025 / IMCP: subsidio al empleo
// 2026 con DOS porcentajes segun vigencia:
//   enero 2026: 15.59% de UMA mensual 2025 ($3,439.46) = $536.21
//   feb-dic 2026: 15.02% de UMA mensual 2026 ($3,566.22) = $535.65
// Fuente de datos: migrations/0002_seed_fiscal_2026.sql (tablas_fiscales).

const SEED: &str = include_str!("../migrations/0002_seed_fiscal_2026.sql");

#[derive(Debug)]
struct FilaSubsidio {
    inicio: String,
    fin: String,
    maximo: f64,
    tope: f64,
}

fn filas_subsidio() -> Vec<FilaSubsidio> {
    let mut out = Vec::new();
    for line in SEED.lines() {
        if !line.contains("'subsidio_empleo'") {
            continue;
        }
        let marker = "'subsidio_empleo', '";
        let pos = line.find(marker).expect("fila subsidio mal formada");
        let rest = &line[pos + marker.len()..];
        // rest = "2026-01-01', '2026-01-31', '{\"maximo\":536.21,...}'"
        let inicio = rest[0..10].to_string();
        let fin = rest[14..24].to_string();
        let json_start = rest.find('{').expect("JSON ausente");
        let json = &rest[json_start..];
        let maximo = json
            .split("\"maximo\":")
            .nth(1)
            .expect("maximo ausente")
            .split(|c| c == ',' || c == '}')
            .next()
            .unwrap()
            .trim()
            .parse::<f64>()
            .unwrap();
        let tope = json
            .split("\"tope\":")
            .nth(1)
            .expect("tope ausente")
            .split(|c| c == ',' || c == '}')
            .next()
            .unwrap()
            .trim()
            .parse::<f64>()
            .unwrap();
        out.push(FilaSubsidio { inicio, fin, maximo, tope });
    }
    out
}

fn subsidio_para_fecha(fecha: &str) -> f64 {
    filas_subsidio()
        .into_iter()
        .find(|f| f.inicio.as_str() <= fecha && fecha <= f.fin.as_str())
        .expect("no hay fila vigente para la fecha")
        .maximo
}

#[test]
fn subsidio_enero_2026_536_21() {
    // Fuente: DOF 31-12-2025, articulo transitorio.
    // manual: 15.59% x 3,439.46 = 536.21 (UMA 2025, enero 2026).
    let filas = filas_subsidio();
    assert_eq!(filas.len(), 2, "deben existir dos filas de subsidio_empleo");
    let v = subsidio_para_fecha("2026-01-15");
    assert!((v - 536.21).abs() < 0.01, "enero debe usar 536.21, obtenido {}", v);
    let enero = filas.iter().find(|f| f.inicio == "2026-01-01").unwrap();
    assert_eq!(enero.fin, "2026-01-31");
    assert!((enero.tope - 11492.66).abs() < 0.01);
}

#[test]
fn subsidio_febrero_2026_535_65() {
    // Fuente: DOF 31-12-2025 / IMCP.
    // manual: 15.02% x 3,566.22 = 535.65 (UMA 2026, feb-dic).
    let v = subsidio_para_fecha("2026-02-15");
    assert!((v - 535.65).abs() < 0.01, "febrero debe usar 535.65, obtenido {}", v);
    let feb = filas_subsidio().into_iter().find(|f| f.inicio == "2026-02-01").unwrap();
    assert_eq!(feb.fin, "2026-12-31");
    assert!((feb.tope - 11492.66).abs() < 0.01);
}

#[test]
fn subsidio_consulta_por_fecha_pago_en_calcular_recibo() {
    // Confirmacion: calcular_recibo usa cargar_param_vigente(tipo, fecha_pago)
    // para el subsidio, por lo que enero y febrero resuelven filas distintas.
    let src = include_str!("../src/commands/nomina.rs");
    assert!(src.contains("cargar_param_vigente(&app, \"subsidio_empleo\", &fecha_pago)"));
    assert!(src.contains("AND vigencia_inicio<=? AND vigencia_fin>=?"));
}
