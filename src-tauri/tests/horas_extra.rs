// Bloque B — evidencia de horas extraordinarias.
// Cada caso incluye el calculo manual y la fuente legal consultada.

use nomina_mx_lib::calc::horas_extra::{
    calcular_exencion, desglosar_horas_extra, integrar_en_sbc, valor_hora_ordinaria,
    ParametrosHorasExtra,
};
use nomina_mx_lib::commands::cfdi_cmd::{
    generar_cfdi, CfdiRequest, ConceptoPercepcion, RegistroHorasExtra,
};
use std::path::PathBuf;
use std::process::Command;

// Copia de la fila vigente 2026 de migrations/0003_horas_extra.sql.
fn params() -> ParametrosHorasExtra {
    ParametrosHorasExtra {
        factor_dobles: 2.0,
        factor_triples: 3.0,
        limite_dobles_horas: 9.0,
        limite_triples_horas: 4.0,
        porcentaje_exento: 0.5,
        tope_exento_umas_semana: 5.0,
        horas_jornada_diurna: 8.0,
        horas_jornada_nocturna: 7.0,
        horas_jornada_mixta: 7.5,
    }
}

fn req_con_horas_extra() -> CfdiRequest {
    CfdiRequest {
        empresa_rfc: "XAXX010101000".into(),
        empresa_razon: "Empresa Demo".into(),
        receptor_rfc: "XEXX010101000".into(),
        receptor_nombre: "Trabajador Demo".into(),
        receptor_curp: "XEXX010101HNEXXXA4".into(),
        receptor_nss: "12345678901".into(),
        fecha_pago: "2026-02-28".into(),
        fecha_inicio: "2026-02-01".into(),
        fecha_fin: "2026-02-28".into(),
        dias_pagados: 30.0,
        total_percepciones: 11350.0,
        total_deducciones: 1000.0,
        neto: 10350.0,
        conceptos_percepcion: vec![
            ConceptoPercepcion {
                tipo: "001".into(),
                clave: "001".into(),
                concepto: "Sueldos y salarios".into(),
                importe_gravado: 10000.0,
                importe_exento: 0.0,
                horas_extra: vec![],
            },
            ConceptoPercepcion {
                tipo: "019".into(),
                clave: "019".into(),
                concepto: "Horas extra".into(),
                importe_gravado: 900.0,
                importe_exento: 450.0,
                horas_extra: vec![
                    RegistroHorasExtra {
                        dias: 3,
                        tipo_horas: "01".into(),
                        horas_reales: 9,
                        importe_pagado: 900.0,
                    },
                    RegistroHorasExtra {
                        dias: 1,
                        tipo_horas: "02".into(),
                        horas_reales: 3,
                        importe_pagado: 450.0,
                    },
                ],
            },
        ],
        conceptos_deduccion: vec![],
    }
}

fn validar_xsd(xml: &str, nombre: &str) -> std::process::Output {
    let path = std::env::temp_dir().join(nombre);
    std::fs::write(&path, xml).unwrap();
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .to_path_buf();
    Command::new("xmllint")
        .env("LANG", "C")
        .env("LC_ALL", "C")
        .args(["--noout", "--schema"])
        .arg(root.join("tests/xsd/all.xsd"))
        .arg(&path)
        .current_dir(root)
        .output()
        .expect("xmllint no disponible")
}

#[test]
fn valor_hora_por_jornada() {
    // Fuente: LFT art. 61 (diurna 8 h, nocturna 7 h, mixta 7.5 h).
    // Manual: 400 / 8 = 50; 400 / 7 = 57.142857...
    assert_eq!(valor_hora_ordinaria(400.0, 8.0).unwrap(), 50.0);
    let nocturna = valor_hora_ordinaria(400.0, 7.0).unwrap();
    assert!((nocturna - 57.142857).abs() < 0.00001);
}

#[test]
fn caso_12_horas_9_dobles_3_triples() {
    // Fuentes: LFT art. 66 (100% adicional = doble), Cuarto Transitorio DOF
    // 01-05-2026 (tope de 9 horas en 2026) y LFT art. 68 (200% adicional = triple).
    // Manual: valor hora 50; 9*50*2 = 900; 3*50*3 = 450; total 1350.
    let d = desglosar_horas_extra(12.0, 400.0, 8.0, &params()).unwrap();
    assert_eq!(d.horas_dobles, 9.0);
    assert_eq!(d.horas_triples, 3.0);
    assert_eq!(d.importe_dobles, 900.0);
    assert_eq!(d.importe_triples, 450.0);
    assert_eq!(d.importe_total, 1350.0);
    assert!(!d.excede_limite_legal);
}

#[test]
fn exencion_isr_art_93_fraccion_i() {
    // Fuente: LISR art. 93 fracc. I (SAT: wwwmat.sat.gob.mx/articulo/15199).
    // Manual (UMA 117.31, 1 semana): 900*50% = 450 < 5*117.31 = 586.55.
    // Gravado = 1350 - 450 = 900. Las horas triples quedan gravadas.
    let e = calcular_exencion(900.0, 450.0, false, 117.31, 1.0, &params());
    assert_eq!(e.exento, 450.0);
    assert_eq!(e.gravado, 900.0);
    assert_eq!(e.tope_aplicado, 586.55);
}

#[test]
fn exencion_se_topea_a_cinco_umas() {
    // Fuente: LISR art. 93 fracc. I; tope con UMA por desindexacion (CPEUM 26 C).
    // Manual: 1800*50% = 900 > 586.55; total 2700; gravado 2113.45.
    let e = calcular_exencion(1800.0, 900.0, false, 117.31, 1.0, &params());
    assert_eq!(e.exento, 586.55);
    assert_eq!(e.gravado, 2113.45);
}

#[test]
fn sbc_incluye_horas_extra_con_tope_de_25_uma() {
    // Fuente: LSS art. 27 (salario variable integra SBC) y art. 28 (tope 25 UMA).
    // Manual: 400 + 1350/30 = 445; tope 25*117.31 = 2932.75.
    assert_eq!(integrar_en_sbc(400.0, 1350.0, 30.0, 117.31).unwrap(), 445.0);
    assert_eq!(integrar_en_sbc(4000.0, 1350.0, 30.0, 117.31).unwrap(), 2932.75);
}

#[test]
fn cfdi_019_emite_horas_extra_y_valida_xsd() {
    // Fuente: catalogo c_TipoHoras del SAT (01 dobles, 02 triples) y
    // complemento Nomina 1.2 (nodo condicional HorasExtra).
    let rt = tokio::runtime::Runtime::new().unwrap();
    let xml = rt.block_on(generar_cfdi(req_con_horas_extra())).unwrap().xml;

    assert!(xml.contains(r#"TipoPercepcion="019""#));
    assert!(xml.contains(r#"TipoHoras="01" HorasExtra="9" ImportePagado="900.00""#));
    assert!(xml.contains(r#"TipoHoras="02" HorasExtra="3" ImportePagado="450.00""#));
    assert!(xml.contains(r#"ImporteGravado="900" ImporteExento="450""#));
    assert!(xml.contains(r#"TotalExento="450""#));
    assert!(xml.contains(r#"TotalSueldos="11350""#));

    let out = validar_xsd(&xml, "cfdi_horas_extra_bloque_b.xml");
    assert!(
        out.status.success(),
        "XSD rechazo el CFDI con HorasExtra: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

#[test]
fn cfdi_horas_extra_rechaza_tipo_horas_invalido() {
    let mut r = req_con_horas_extra();
    r.conceptos_percepcion[1].horas_extra[0].tipo_horas = "99".into();
    let rt = tokio::runtime::Runtime::new().unwrap();
    let err = rt.block_on(generar_cfdi(r)).unwrap_err();
    assert!(err.contains("TipoHoras invalido"), "{}", err);
}

#[test]
fn cfdi_horas_extra_rechaza_importe_no_cuadrado() {
    let mut r = req_con_horas_extra();
    r.conceptos_percepcion[1].horas_extra[0].importe_pagado = 899.0;
    let rt = tokio::runtime::Runtime::new().unwrap();
    let err = rt.block_on(generar_cfdi(r)).unwrap_err();
    assert!(err.contains("suma ImportePagado"), "{}", err);
}

#[test]
fn cfdi_horas_extra_rechaza_concepto_que_no_sea_019() {
    let mut r = req_con_horas_extra();
    r.conceptos_percepcion[0].horas_extra = r.conceptos_percepcion[1].horas_extra.clone();
    let rt = tokio::runtime::Runtime::new().unwrap();
    let err = rt.block_on(generar_cfdi(r)).unwrap_err();
    assert!(err.contains("solo aplica a TipoPercepcion=019"), "{}", err);
}
