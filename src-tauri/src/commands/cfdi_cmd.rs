use serde::{Deserialize, Serialize};

/// Nodo condicional `nomina12:HorasExtra` (catalogo `c_TipoHoras` del SAT):
/// 01 = dobles, 02 = triples, 03 = simples.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RegistroHorasExtra {
    pub dias: i64,
    pub tipo_horas: String,
    pub horas_reales: i64,
    pub importe_pagado: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ConceptoPercepcion {
    pub tipo: String,
    pub clave: String,
    pub concepto: String,
    pub importe_gravado: f64,
    pub importe_exento: f64,
    // Bloque B: solo aplica cuando TipoPercepcion="019" (Horas extra).
    #[serde(default)]
    pub horas_extra: Vec<RegistroHorasExtra>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ConceptoDeduccion {
    pub tipo: String,
    pub clave: String,
    pub concepto: String,
    pub importe: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CfdiRequest {
    pub empresa_rfc: String,
    pub empresa_razon: String,
    pub receptor_rfc: String,
    pub receptor_nombre: String,
    pub receptor_curp: String,
    pub receptor_nss: String,
    pub fecha_pago: String,
    pub fecha_inicio: String,
    pub fecha_fin: String,
    pub dias_pagados: f64,
    pub total_percepciones: f64,
    pub total_deducciones: f64,
    pub neto: f64,
    // A6: conceptos semánticos. Si estan vacios se usa el recibo de
    // sueldo ordinario por compatibilidad (001 / 002).
    #[serde(default)]
    pub conceptos_percepcion: Vec<ConceptoPercepcion>,
    #[serde(default)]
    pub conceptos_deduccion: Vec<ConceptoDeduccion>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CfdiResult {
    pub xml: String,
    pub uuid: Option<String>,
}

fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

// P1: TotalSueldos = suma de (gravado + exento) de los conceptos cuyo
// TipoPercepcion NO sea 022, 023, 025, 039 ni 044 (guia SAT / SIFEI).
// TotalGravado y TotalExento se suman sobre TODOS los conceptos.
fn totales_percepciones(req: &CfdiRequest) -> (f64, f64, f64) {
    let round2 = |v: f64| (v * 100.0).round() / 100.0;
    if req.conceptos_percepcion.is_empty() {
        return (round2(req.total_percepciones), round2(req.total_percepciones), 0.0);
    }
    let excluidos = ["022", "023", "025", "039", "044"];
    let gravado: f64 = req.conceptos_percepcion.iter().map(|c| c.importe_gravado).sum();
    let exento: f64 = req.conceptos_percepcion.iter().map(|c| c.importe_exento).sum();
    let sueldos: f64 = req
        .conceptos_percepcion
        .iter()
        .filter(|c| !excluidos.contains(&c.tipo.as_str()))
        .map(|c| c.importe_gravado + c.importe_exento)
        .sum();
    (round2(sueldos), round2(gravado), round2(exento))
}

// Bloque B: nodos `HorasExtra` del complemento Nomina 1.2. El importe se
// formatea a dos decimales porque el tipo tdCFDI:t_ImporteMXN lo exige.
fn build_horas_extra(c: &ConceptoPercepcion) -> String {
    if c.horas_extra.is_empty() {
        return String::new();
    }
    let items = c
        .horas_extra
        .iter()
        .map(|h| {
            format!(
                r#"<nomina12:HorasExtra Dias="{}" TipoHoras="{}" HorasExtra="{}" ImportePagado="{:.2}"/>"#,
                h.dias,
                h.tipo_horas,
                h.horas_reales,
                (h.importe_pagado * 100.0).round() / 100.0,
            )
        })
        .collect::<Vec<_>>()
        .join("\n          ");
    format!("\n          {}\n        ", items)
}

fn build_percepciones(req: &CfdiRequest) -> String {
    let (sueldos, gravado, exento_total) = totales_percepciones(req);
    let total = req.total_percepciones;
    let items: String = if req.conceptos_percepcion.is_empty() {
        format!(
            r#"<nomina12:Percepcion TipoPercepcion="001" Clave="001" Concepto="Sueldos y salarios" ImporteGravado="{total}" ImporteExento="0"/>"#
        )
    } else {
        req.conceptos_percepcion
            .iter()
            .map(|c| {
                let hijos = build_horas_extra(c);
                if hijos.is_empty() {
                    format!(
                        r#"<nomina12:Percepcion TipoPercepcion="{tipo}" Clave="{clave}" Concepto="{concepto}" ImporteGravado="{gravado}" ImporteExento="{exento}"/>"#,
                        tipo = c.tipo,
                        clave = c.clave,
                        concepto = escape_xml(&c.concepto),
                        gravado = c.importe_gravado,
                        exento = c.importe_exento,
                    )
                } else {
                    format!(
                        r#"<nomina12:Percepcion TipoPercepcion="{tipo}" Clave="{clave}" Concepto="{concepto}" ImporteGravado="{gravado}" ImporteExento="{exento}">{hijos}</nomina12:Percepcion>"#,
                        tipo = c.tipo,
                        clave = c.clave,
                        concepto = escape_xml(&c.concepto),
                        gravado = c.importe_gravado,
                        exento = c.importe_exento,
                        hijos = hijos,
                    )
                }
            })
            .collect::<Vec<_>>()
            .join("\n        ")
    };
    format!(
        r#"<nomina12:Percepciones TotalSueldos="{sueldos}" TotalGravado="{gravado}" TotalExento="{exento}">
        {items}
      </nomina12:Percepciones>"#,
        sueldos = sueldos,
        gravado = gravado,
        exento = exento_total,
    )
}

fn build_deducciones(req: &CfdiRequest) -> String {
    let total = req.total_deducciones;
    let items: String = if req.conceptos_deduccion.is_empty() {
        format!(
            r#"<nomina12:Deduccion TipoDeduccion="002" Clave="002" Concepto="ISR" Importe="{total}"/>"#
        )
    } else {
        req.conceptos_deduccion
            .iter()
            .map(|c| {
                format!(
                    r#"<nomina12:Deduccion TipoDeduccion="{tipo}" Clave="{clave}" Concepto="{concepto}" Importe="{importe}"/>"#,
                    tipo = c.tipo,
                    clave = c.clave,
                    concepto = escape_xml(&c.concepto),
                    importe = c.importe,
                )
            })
            .collect::<Vec<_>>()
            .join("\n        ")
    };
    format!(
        r#"<nomina12:Deducciones TotalOtrasDeducciones="0" TotalImpuestosRetenidos="{total}">
        {items}
      </nomina12:Deducciones>"#
    )
}

#[tauri::command]
pub async fn generar_cfdi(req: CfdiRequest) -> Result<CfdiResult, String> {
    // P2: consistencia entre totales declarados y suma de conceptos.
    if !req.conceptos_percepcion.is_empty() {
        let suma: f64 = req
            .conceptos_percepcion
            .iter()
            .map(|c| c.importe_gravado + c.importe_exento)
            .sum();
        if (suma - req.total_percepciones).abs() > 0.01 {
            return Err(format!(
                "total_percepciones ({}) != suma de conceptos ({})",
                req.total_percepciones, suma
            ));
        }
    }
    if !req.conceptos_deduccion.is_empty() {
        let suma: f64 = req.conceptos_deduccion.iter().map(|c| c.importe).sum();
        if (suma - req.total_deducciones).abs() > 0.01 {
            return Err(format!(
                "total_deducciones ({}) != suma de conceptos ({})",
                req.total_deducciones, suma
            ));
        }
    }

    // Bloque B: integridad de los nodos HorasExtra (catalogo c_TipoHoras).
    for c in req.conceptos_percepcion.iter().filter(|c| !c.horas_extra.is_empty()) {
        if c.tipo != "019" {
            return Err(format!(
                "HorasExtra solo aplica a TipoPercepcion=019 (se recibio {})",
                c.tipo
            ));
        }
        let suma: f64 = c.horas_extra.iter().map(|h| h.importe_pagado).sum();
        if (suma - (c.importe_gravado + c.importe_exento)).abs() > 0.01 {
            return Err(format!(
                "HorasExtra: suma ImportePagado ({}) != ImporteGravado+ImporteExento ({})",
                suma,
                c.importe_gravado + c.importe_exento
            ));
        }
        for h in c.horas_extra.iter() {
            if !["01", "02", "03"].contains(&h.tipo_horas.as_str()) {
                return Err(format!("TipoHoras invalido: {} (use 01, 02 o 03)", h.tipo_horas));
            }
            if h.dias < 1 || h.horas_reales < 1 || h.importe_pagado < 0.0 {
                return Err("HorasExtra: Dias y HorasExtra deben ser >= 1 e ImportePagado >= 0".to_string());
            }
        }
    }

    // P1: TotalPercepciones del complemento = TotalSueldos calculado.
    let (total_sueldos, _gravado, _exento) = totales_percepciones(&req);
    // Genera XML CFDI 4.0 con complemento Nomina 1.2 (estructura basica).
    let percepciones = build_percepciones(&req);
    let deducciones = build_deducciones(&req);
    let xml = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<cfdi:Comprobante xmlns:cfdi="http://www.sat.gob.mx/cfd/4" xmlns:nomina12="http://www.sat.gob.mx/nomina12" Version="4.0" TipoDeComprobante="N" MetodoPago="PUE" FormaPago="99" Exportacion="01" Fecha="{fecha}T12:00:00" SubTotal="{sub}" Descuento="{td_attr}" Total="{total}" Moneda="MXN" LugarExpedicion="01000" NoCertificado="00000000000000000000" Certificado="MIIBkTCB+wIBADANBgkqhkiG9w0BAQQFADBPMQswCQYDVQQGEwJNWDEVMBMGA1UECBMMQ2l1ZGFkIGRlIE1leGljbzEXMBUGA1UEChMOTm9taW5hTVggU0FOREJPWAgGA1UEAxMUTk9NSU5BWFhODMwOTEzRU4xMA0GCSqGSIb3DQEBAQUAA4GMADCBiAKBgF8Q3p7X2pQ9xQZb4xYmS8kL1vN6T0hD7mD3Q0JZ8U5p2nK6sYcV4bX1nF0aE9rT7yD2uW5mQ4pL8vC3xN1bZ6sF0hJ7kG9mD2xR5tY8uI1qE4wA7bN0cV6sF3hJ9kL2mD5xR8tY1uI4qE7wAgMBAAE=" Sello="aGVsbG8gd29ybGQgc2VsbG8gc2FuZGJveCBub21pbmFteCBkZW1vIHNpZ25hdHVyZQ==">
  <cfdi:Emisor Rfc="{erfc}" Nombre="{erazon}" RegimenFiscal="601"/>
  <cfdi:Receptor Rfc="{rrfc}" Nombre="{rnombre}" DomicilioFiscalReceptor="01000" RegimenFiscalReceptor="605" UsoCFDI="CN01"/>
  <cfdi:Conceptos>
    <cfdi:Concepto ClaveProdServ="84111505" Cantidad="1" ClaveUnidad="ACT" Descripcion="Nomina" ValorUnitario="{sub}" Importe="{sub}" ObjetoImp="01"/>
  </cfdi:Conceptos>
  <cfdi:Complemento>
    <nomina12:Nomina Version="1.2" TipoNomina="O" FechaPago="{fp}" FechaInicialPago="{fi}" FechaFinalPago="{ff}" NumDiasPagados="{dias}" TotalPercepciones="{tp}" TotalDeducciones="{td}" TotalOtrosPagos="0">
      <nomina12:Receptor Curp="{curp}" NumSeguridadSocial="{nss}" FechaInicioRelLaboral="2020-01-01" Antigüedad="P1W" TipoContrato="01" TipoJornada="01" TipoRegimen="02" NumEmpleado="1" Departamento="Gral" Puesto="Empleado" RiesgoPuesto="1" PeriodicidadPago="04" SalarioBaseCotApor="0" SalarioDiarioIntegrado="0" ClaveEntFed="CMX"/>
      {percepciones}
      {deducciones}
    </nomina12:Nomina>
  </cfdi:Complemento>
</cfdi:Comprobante>"#,
        fecha = req.fecha_pago,
        sub = req.total_percepciones,
        total = req.neto,
        erfc = req.empresa_rfc,
        erazon = req.empresa_razon,
        rrfc = req.receptor_rfc,
        rnombre = req.receptor_nombre,
        fp = req.fecha_pago,
        fi = req.fecha_inicio,
        ff = req.fecha_fin,
        dias = req.dias_pagados,
        tp = total_sueldos,
        td = req.total_deducciones,
        td_attr = req.total_deducciones,
        curp = req.receptor_curp,
        nss = req.receptor_nss,
        percepciones = percepciones,
        deducciones = deducciones,
    );
    // En sandbox real se enviaria a PAC; aqui retornamos xml sin timbrar.
    Ok(CfdiResult { xml, uuid: None })
}
