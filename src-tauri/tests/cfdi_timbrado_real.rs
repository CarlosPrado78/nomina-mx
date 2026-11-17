// Test de timbrado REAL con CSD y PAC. Solo se ejecuta con
// `cargo test -- --ignored` cuando el entorno tiene:
//   - CSD_EMISOR_CER: ruta al .cer del emisor
//   - CSD_EMISOR_KEY: ruta al .key del emisor
//   - CSD_EMISOR_PASS: contraseña del .key
//   - PAC_URL: endpoint del PAC (sandbox o prod)
//   - PAC_USER, PAC_PASS: credenciales
//
// Sin estas variables, el test se salta con #[ignore].
//
// Cuando se implemente la firma real, este test debe:
// 1. Cargar CSD del emisor.
// 2. Generar cadena original del CFDI (XSLT oficial del SAT).
// 3. Sellar con RSA-SHA256 usando la llave privada.
// 4. Embeber Sello, Certificado, NoCertificado.
// 5. Enviar al PAC.
// 6. Verificar UUID, estatus "Timbrado", y que el XML timbrado
//    se pueda validar contra XSD.

#[test]
#[ignore = "Requiere CSD y PAC configurados; ver docs/smoke_test.md"]
fn timbrado_real_con_pac_sandbox() {
    // TODO: implementar en Fase 10 cuando haya CSD y PAC.
    panic!("Implementar timbrado real. Ver instrucciones en comentario.");
}
