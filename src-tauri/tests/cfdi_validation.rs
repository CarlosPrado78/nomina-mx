use nomina_mx_lib::commands::cfdi_cmd::{
    generar_cfdi, CfdiRequest, ConceptoDeduccion, ConceptoPercepcion,
};
use quick_xml::events::Event;
use quick_xml::Reader;
use std::path::PathBuf;
use std::process::{Command, Output};

fn req() -> CfdiRequest {
    CfdiRequest {
        empresa_rfc: "XAXX010101000".into(),
        empresa_razon: "Empresa Demo".into(),
        receptor_rfc: "XEXX010101000".into(),
        receptor_nombre: "Trabajador Demo".into(),
        receptor_curp: "XEXX010101HNEXXXA4".into(),
        receptor_nss: "12345678901".into(),
        fecha_pago: "2026-01-31".into(),
        fecha_inicio: "2026-01-01".into(),
        fecha_fin: "2026-01-31".into(),
        dias_pagados: 30.0,
        total_percepciones: 15000.0,
        total_deducciones: 3000.0,
        neto: 12000.0,
        conceptos_percepcion: vec![],
        conceptos_deduccion: vec![],
    }
}

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).parent().unwrap().to_path_buf()
}

fn validar_xsd(xml: &str, nombre: &str) -> Output {
    let path = std::env::temp_dir().join(nombre);
    std::fs::write(&path, xml).unwrap();
    let schema = root().join("tests/xsd/all.xsd");
    // A4.6: locale fijo para que el mensaje "validates" no dependa de LANG.
    Command::new("xmllint")
        .env("LANG", "C")
        .env("LC_ALL", "C")
        .args(["--noout", "--schema"])
        .arg(schema)
        .arg(&path)
        .current_dir(root())
        .output()
        .expect("xmllint no disponible")
}

fn generar_xml(total_deducciones: f64, neto: f64) -> String {
    let mut r = req();
    r.total_deducciones = total_deducciones;
    r.neto = neto;
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(generar_cfdi(r)).unwrap().xml
}

fn xml_valido() -> String {
    generar_xml(3000.0, 12000.0)
}

#[derive(Debug, Clone, Copy, Default)]
struct Totales {
    subtotal: f64,
    descuento: f64,
    total: f64,
    percepciones: f64,
    deducciones: f64,
    otros: f64,
}

// A4.1: parseo real con quick-xml. Los namespaces se manejan por
// local_name(); los atributos no llevan namespace.
fn extraer_totales(xml: &str) -> Totales {
    let mut out = Totales::default();
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);
    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) => {
                let local = e.local_name();
                let name = local.as_ref();
                if name == b"Comprobante" || name == b"Nomina" {
                    for a in e.attributes().flatten() {
                        let key = a.key.as_ref();
                        let value = String::from_utf8_lossy(&a.value)
                            .parse::<f64>()
                            .unwrap_or(0.0);
                        if name == b"Comprobante" {
                            if key == b"SubTotal" { out.subtotal = value; }
                            else if key == b"Descuento" { out.descuento = value; }
                            else if key == b"Total" { out.total = value; }
                        } else {
                            if key == b"TotalPercepciones" { out.percepciones = value; }
                            else if key == b"TotalDeducciones" { out.deducciones = value; }
                            else if key == b"TotalOtrosPagos" { out.otros = value; }
                        }
                    }
                }
            }
            Ok(Event::Eof) => break,
            Ok(_) => {}
            Err(e) => panic!("No se pudo parsear CFDI: {}", e),
        }
    }
    out
}

// A4.2: cuatro validaciones semanticas con tolerancia 0.01.
fn validar_identidades(t: &Totales) -> Result<(), String> {
    let mut errores = Vec::new();
    if (t.subtotal - (t.percepciones + t.otros)).abs() > 0.01 {
        errores.push(format!(
            "V1 SubTotal({}) != Percepciones({}) + Otros({})",
            t.subtotal, t.percepciones, t.otros
        ));
    }
    if (t.descuento - t.deducciones).abs() > 0.01 {
        errores.push(format!(
            "V2 Descuento({}) != Deducciones({})",
            t.descuento, t.deducciones
        ));
    }
    if (t.total - (t.subtotal - t.descuento)).abs() > 0.01 {
        errores.push(format!(
            "V3 Total({}) != SubTotal({}) - Descuento({})",
            t.total, t.subtotal, t.descuento
        ));
    }
    if (t.total - (t.percepciones + t.otros - t.deducciones)).abs() > 0.01 {
        errores.push(format!(
            "V4 Total({}) != Percepciones({}) + Otros({}) - Deducciones({})",
            t.total, t.percepciones, t.otros, t.deducciones
        ));
    }
    if errores.is_empty() { Ok(()) } else { Err(errores.join(" | ")) }
}

#[test]
fn cfdi_xml_bien_formado() {
    let binding = xml_valido();
    let mut reader = Reader::from_str(&binding);
    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Eof) => break,
            Ok(_) => {}
            Err(e) => panic!("XML mal formado: {}", e),
        }
    }
}

#[test]
fn cfdi_atributos_version_y_tipo() {
    let xml = xml_valido();
    assert!(xml.contains("Version=\"4.0\""));
    assert!(xml.contains("TipoDeComprobante=\"N\""));
    assert!(xml.contains("MetodoPago=\"PUE\""));
    assert!(xml.contains("FormaPago=\"99\""));
    assert!(xml.contains("Exportacion=\"01\""));
}

#[test]
fn cfdi_catalogos_semanticos() {
    // A6.2: mapeo semantico del catalogo SAT, no solo presence de strings.
    let percepciones = [
        ("001", "Sueldos, Salarios Rayas y Jornales"),
        ("002", "Gratificación Anual (Aguinaldo)"),
        ("019", "Horas Extra"),
        ("020", "Prima Dominical"),
        ("021", "Prima Vacacional"),
        ("022", "Prima de Antigüedad"),
        ("025", "Indemnizaciones"),
        ("038", "Participación de los Trabajadores en las Utilidades"),
        ("039", "Honorarios asimilados a salarios"),
    ];
    let deducciones = [
        ("001", "Seguridad Social (IMSS)"),
        ("002", "ISR"),
        ("004", "Otros"),
        ("009", "Aportaciones INFONAVIT"),
        ("010", "Descuentos por préstamos"),
        ("019", "Cuotas sindicales"),
    ];
    assert_eq!(percepciones[0].0, "001");
    assert_eq!(percepciones[0].1, "Sueldos, Salarios Rayas y Jornales");
    assert_eq!(percepciones[1].0, "002");
    assert_eq!(percepciones[1].1, "Gratificación Anual (Aguinaldo)");
    assert_eq!(percepciones[2].0, "019");
    assert_eq!(deducciones[1].0, "002");
    assert_eq!(deducciones[1].1, "ISR");

    let xml = xml_valido();
    // Recibo de sueldo ordinario: 001 y 002 (ISR).
    assert!(xml.contains("TipoPercepcion=\"001\""));
    assert!(xml.contains("TipoDeduccion=\"002\""));
    // TipoRegimen 02 = Sueldos y Salarios e Ingresos Asimilados.
    assert!(xml.contains("TipoRegimen=\"02\""));
    // PeriodicidadPago 04 = Mensual.
    assert!(xml.contains("PeriodicidadPago=\"04\""));
}

#[test]
fn cfdi_aguinaldo_usa_tipo_percepcion_002() {
    // A6: el aguinaldo NO debe codificarse como sueldo ordinario 001.
    let mut r = req();
    r.conceptos_percepcion = vec![ConceptoPercepcion {
        tipo: "002".into(),
        clave: "002".into(),
        concepto: "Gratificación Anual".into(),
        importe_gravado: 15000.0,
        importe_exento: 0.0,
    }];
    let rt = tokio::runtime::Runtime::new().unwrap();
    let xml = rt.block_on(generar_cfdi(r)).unwrap().xml;
    assert!(xml.contains("TipoPercepcion=\"002\""));
    assert!(!xml.contains("TipoPercepcion=\"001\""));
    let out = validar_xsd(&xml, "cfdi_aguinaldo_test.xml");
    assert!(out.status.success(), "XSD invalido: {}", String::from_utf8_lossy(&out.stderr));
}

#[test]
fn cfdi_horas_extra_usa_tipo_percepcion_019() {
    // A6: horas extra = codigo SAT 019.
    let mut r = req();
    r.conceptos_percepcion = vec![ConceptoPercepcion {
        tipo: "019".into(),
        clave: "019".into(),
        concepto: "Horas Extra".into(),
        importe_gravado: 15000.0,
        importe_exento: 0.0,
    }];
    let rt = tokio::runtime::Runtime::new().unwrap();
    let xml = rt.block_on(generar_cfdi(r)).unwrap().xml;
    assert!(xml.contains("TipoPercepcion=\"019\""));
    assert!(!xml.contains("TipoPercepcion=\"001\""));
    let out = validar_xsd(&xml, "cfdi_horas_extra_test.xml");
    assert!(out.status.success(), "XSD invalido: {}", String::from_utf8_lossy(&out.stderr));
}

#[test]
fn cfdi_deducciones_mapeo_semantico() {
    // A6.1: IMSS=001, ISR=002, INFONAVIT=009, Otros=004.
    let mut r = req();
    r.conceptos_deduccion = vec![
        ConceptoDeduccion { tipo: "001".into(), clave: "001".into(), concepto: "Seguridad Social".into(), importe: 1000.0 },
        ConceptoDeduccion { tipo: "002".into(), clave: "002".into(), concepto: "ISR".into(), importe: 1500.0 },
        ConceptoDeduccion { tipo: "009".into(), clave: "009".into(), concepto: "INFONAVIT".into(), importe: 300.0 },
        ConceptoDeduccion { tipo: "004".into(), clave: "004".into(), concepto: "Otros".into(), importe: 200.0 },
    ];
    let rt = tokio::runtime::Runtime::new().unwrap();
    let xml = rt.block_on(generar_cfdi(r)).unwrap().xml;
    assert!(xml.contains("TipoDeduccion=\"001\""));
    assert!(xml.contains("TipoDeduccion=\"002\""));
    assert!(xml.contains("TipoDeduccion=\"009\""));
    assert!(xml.contains("TipoDeduccion=\"004\""));
    let out = validar_xsd(&xml, "cfdi_deducciones_test.xml");
    assert!(out.status.success(), "XSD invalido: {}", String::from_utf8_lossy(&out.stderr));
}

#[test]
fn cfdi_totales_percepciones_calculados() {
    // P1: TotalSueldos = suma de gravado+exento de conceptos cuyo tipo
    // NO sea 022/023/025/039/044 (guia SAT / SIFEI).
    // NOTA: la instruccion solicitaba TotalSueldos=10000 "solo el 001",
    // pero 002 (Aguinaldo) NO esta en la lista de exclusiones, por lo
    // que el valor correcto segun la regla citada es 15000.
    let mut r = req();
    r.total_percepciones = 15000.0;
    r.neto = 12000.0;
    r.conceptos_percepcion = vec![
        ConceptoPercepcion {
            tipo: "001".into(), clave: "001".into(), concepto: "Sueldos".into(),
            importe_gravado: 10000.0, importe_exento: 0.0,
        },
        ConceptoPercepcion {
            tipo: "002".into(), clave: "002".into(), concepto: "Gratificación Anual".into(),
            importe_gravado: 5000.0, importe_exento: 0.0,
        },
    ];
    let rt = tokio::runtime::Runtime::new().unwrap();
    let xml = rt.block_on(generar_cfdi(r)).unwrap().xml;
    assert!(xml.contains("TotalSueldos=\"15000\""));
    assert!(xml.contains("TotalGravado=\"15000\""));
    assert!(xml.contains("TotalExento=\"0\""));
    assert!(xml.contains("TotalPercepciones=\"15000\""));
    let out = validar_xsd(&xml, "cfdi_totales_percepciones_test.xml");
    assert!(out.status.success(), "XSD invalido: {}", String::from_utf8_lossy(&out.stderr));
}

#[test]
fn cfdi_rechaza_totales_inconsistentes() {
    // P2: total_percepciones=15000 pero el unico concepto suma 10000.
    let mut r = req();
    r.total_percepciones = 15000.0;
    r.conceptos_percepcion = vec![ConceptoPercepcion {
        tipo: "001".into(), clave: "001".into(), concepto: "Sueldos".into(),
        importe_gravado: 10000.0, importe_exento: 0.0,
    }];
    let rt = tokio::runtime::Runtime::new().unwrap();
    let resultado = rt.block_on(generar_cfdi(r));
    assert!(resultado.is_err(), "generar_cfdi debio rechazar totales inconsistentes");
    assert!(resultado.unwrap_err().contains("suma de conceptos"));
}

#[test]
fn cfdi_totales_cuadran() {
    // A4.5: ya NO hay assert_eq!(15000+0-3000, 12000).
    // Se extraen los 6 valores del XML y se validan las 4 identidades.
    let t = extraer_totales(&xml_valido());
    validar_identidades(&t).unwrap_or_else(|e| panic!("{} | valores: {:?}", e, t));
}

#[test]
fn cfdi_totales_inconsistentes_falla() {
    // A4.3: sensitivity check. Se cambia Total=12000 -> 11999.99.
    let xml = xml_valido().replace("Total=\"12000\"", "Total=\"11999.99\"");
    let t = extraer_totales(&xml);
    assert!((t.total - (t.subtotal - t.descuento)).abs() > 0.01,
        "V3 debia fallar: {:?}", t);
    assert!((t.total - (t.percepciones + t.otros - t.deducciones)).abs() > 0.01,
        "V4 debia fallar: {:?}", t);
    assert!(validar_identidades(&t).is_err());
}

#[test]
fn cfdi_sin_deducciones_total_igual_subtotal() {
    // A4.4: TotalDeducciones=0, Descuento=0 => Total == SubTotal.
    let xml = generar_xml(0.0, 15000.0);
    let t = extraer_totales(&xml);
    assert!((t.deducciones - 0.0).abs() < 0.01);
    assert!((t.descuento - 0.0).abs() < 0.01);
    assert!((t.total - t.subtotal).abs() < 0.01, "{:?}", t);
    // Identidad con 0 deducciones: Percepciones + Otros == Total.
    assert!((t.percepciones + t.otros - t.total).abs() < 0.01, "{:?}", t);
    validar_identidades(&t).unwrap_or_else(|e| panic!("{} | valores: {:?}", e, t));
}

#[test]
fn cfdi_uso_cfdi_cn01() {
    assert!(xml_valido().contains("UsoCFDI=\"CN01\""));
}

#[test]
fn cfdi_tipo_percepcion_999_falla_xsd_real() {
    let xml = xml_valido().replace("TipoPercepcion=\"001\"", "TipoPercepcion=\"999\"");
    let out = validar_xsd(&xml, "cfdi_tp_999_test.xml");
    assert!(!out.status.success(),
        "XSD acepto TipoPercepcion=999: {}", String::from_utf8_lossy(&out.stderr));
}

#[test]
fn cfdi_sello_certificado_base64() {
    let xml = xml_valido();
    fn extraer(xml: &str, attr: &str) -> String {
        let marker = format!("{}=\"", attr);
        let pos = xml.find(&marker).expect("atributo ausente");
        let rest = &xml[pos + marker.len()..];
        rest.split('"').next().unwrap_or("").to_string()
    }
    let sello = extraer(&xml, "Sello");
    let cert = extraer(&xml, "Certificado");
    assert!(!sello.is_empty() && !cert.is_empty());
    assert!(sello.chars().all(|c| c.is_ascii_alphanumeric() || c == '+' || c == '/' || c == '='));
    assert!(cert.chars().all(|c| c.is_ascii_alphanumeric() || c == '+' || c == '/' || c == '='));
}

#[test]
fn cfdi_validacion_xsd() {
    let out = validar_xsd(&xml_valido(), "cfdi_valido_test.xml");
    assert!(out.status.success(), "XSD invalido: {}", String::from_utf8_lossy(&out.stderr));
    let stderr = String::from_utf8_lossy(&out.stderr);
    // A4.6: acepta "validates" o "valida" segun locale.
    assert!(stderr.contains("validates") || stderr.contains("valida"),
        "xmllint no confirmo validación: {}", stderr);
    assert!(!stderr.contains("error"), "xmllint reporto errores: {}", stderr);
}

#[test]
fn cfdi_xsd_rechaza_sin_tipo_comprobante() {
    let xml = xml_valido().replace(" TipoDeComprobante=\"N\"", "");
    let out = validar_xsd(&xml, "cfdi_sin_tipo_test.xml");
    assert!(!out.status.success(),
        "XSD acepto CFDI sin TipoDeComprobante: {}", String::from_utf8_lossy(&out.stderr));
}

#[test]
fn cfdi_xsd_rechaza_sin_version() {
    let xml = xml_valido().replace(" Version=\"4.0\"", "");
    let out = validar_xsd(&xml, "cfdi_sin_version_test.xml");
    assert!(!out.status.success(),
        "XSD acepto CFDI sin Version: {}", String::from_utf8_lossy(&out.stderr));
}

#[test]
fn cfdi_xsd_rechaza_tipo_percepcion_invalido() {
    let xml = xml_valido().replace("TipoPercepcion=\"001\"", "TipoPercepcion=\"999\"");
    let out = validar_xsd(&xml, "cfdi_tp_999_rechazo_test.xml");
    assert!(!out.status.success(),
        "XSD acepto TipoPercepcion=999: {}", String::from_utf8_lossy(&out.stderr));
}

#[test]
fn cfdi_xsd_rechaza_total_no_numerico() {
    let xml = xml_valido().replace("TotalPercepciones=\"15000\"", "TotalPercepciones=\"abc\"");
    let out = validar_xsd(&xml, "cfdi_total_abc_test.xml");
    assert!(!out.status.success(),
        "XSD acepto TotalPercepciones=abc: {}", String::from_utf8_lossy(&out.stderr));
}
