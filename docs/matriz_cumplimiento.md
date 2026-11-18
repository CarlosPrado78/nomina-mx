# Matriz de cumplimiento legal

Estado: **Cubierto** = archivo + función + test verde. **Parcial** = existe función/archivo pero falta test dedicado. **Pendiente** = no existe implementación.

| Ley | Artículo | Requisito | Archivo | Función | Test | Estado |
|-----|----------|-----------|---------|---------|------|--------|
| LFT | 76 | Vacaciones por antigüedad | calc/vacaciones.rs | dias_vacaciones_por_antiguedad | vac_anio_01..20 | Cubierto |
| LFT | 80 | Prima vacacional 25% | calc/vacaciones.rs | calcular_prima_vacacional | prima_vac_25 | Cubierto |
| LFT | 87 | Aguinaldo 15 días | calc/aguinaldo.rs | calcular_aguinaldo_proporcional | aguinaldo_180 | Cubierto |
| LFT | 89 | Prima de antigüedad (despido) | calc/liquidacion.rs | calcular_liquidacion | prima_antiguedad_tope_2_sm | Cubierto |
| LFT | 66, 67-68 | Horas extra: dobles 9 h y triples (2026) | calc/horas_extra.rs, migrations/0003_horas_extra.sql | desglosar_horas_extra, horas_jornada_diaria | horas_extra.rs: caso_12_horas_9_dobles_3_triples, trece_horas_es_limite_y_catorce_excede | Cubierto — DOF 01-05-2026, Cuarto Transitorio: 9 h en 2026 |
| LFT | 117-131 | PTU | - | - | - | Pendiente |
| LFT | 162 | Salario mínimo | migrations/0002_seed | (parámetro BD) | caso_1_salario_minimo_mensual | Parcial |
| LFT | 501 | Finiquito/liquidación | calc/finiquito.rs, liquidacion.rs | calcular_finiquito, calcular_liquidacion | finiquito_total, liquidacion_5_anios | Cubierto |
| LSS | 15/17 | Días de cotización no se reducen por incapacidad | calc/imss.rs | calcular_cuotas_imss | incapacidad_no_reduce_dias_cotizados, incapacidad_no_reduce_cuotas_patronales | Cubierto |
| LSS | 27 | SBC no cambia por incapacidad | calc/imss.rs, calc/sdi.rs | calcular_cuotas_imss, calcular_sdi | sbc_no_cambia_por_incapacidad, sdi_tope_25_uma | Cubierto |
| LSS | 29 | Cuotas se causan durante incapacidad | calc/imss.rs | calcular_cuotas_imss | incapacidad_no_reduce_cuotas_obreras, incapacidad_no_reduce_cuotas_patronales | Cubierto |
| LSS | 27, 28 | Horas extra integran SBC (tope 25 UMA) | calc/horas_extra.rs | integrar_en_sbc | horas_extra.rs: sbc_incluye_horas_extra_con_tope_de_25_uma | Cubierto |
| LSS | 106 | Riesgos de trabajo (prima) | calc/imss.rs | calcular_cuotas_imss | imss_patronal_mayor | Parcial |
| LSS | 107 | Enfermedad y maternidad | calc/imss.rs | calcular_cuotas_imss | imss_obrero_positivo | Cubierto |
| LSS | 168 | Cesantía patronal (tabla transición 2026) | calc/imss.rs | tasa_cesantia_patronal_2026, calcular_cesantia_patronal | tests/cesantia_2026.rs (14 tests) | Cubierto — tabla transición 2026, máx 7.513%, objetivo 2030 = 11.875% |
| LISR | 93 fracc. I | Exención ISR de horas extra (50%, tope 5 UMA/semana; 100% salario mínimo) | calc/horas_extra.rs, migrations/0003_horas_extra.sql | calcular_exencion | horas_extra.rs: exencion_isr_art_93_fraccion_i, exencion_se_topea_a_cinco_umas | Cubierto — fuente SAT art. 93; UMA por desindexación (CPEUM 26 C) |
| LISR | 96 | Retención ISR | calc/isr.rs | calcular_isr_neto | isr_tramo_01..11 | Cubierto |
| LISR | 150 | Ajuste anual | - | - | - | Pendiente |
| RLISR | 174 | Retención mensual | calc/isr.rs | calcular_isr_bruto | isr_tramo_03 | Cubierto |
| Decreto SE | DOF 01-05-2024 | Subsidio sin excedente | calc/isr.rs | calcular_isr_neto | subsidio_no_excedente_bajo | Cubierto |
| INFONAVIT | 29 | Aportación 5% SBC | calc/infonavit.rs | calcular_infonavit | infonavit_aport_5pct | Cubierto |
| LFT | 97 | Descuento INFONAVIT tope 20% SM | calc/infonavit.rs | calcular_infonavit | infonavit_tope_20pct | Cubierto |
| Reforma INF | DOF feb-2025 | Descuento en incapacidad | calc/infonavit.rs | calcular_infonavit_con_incapacidad | caso_c_infonavit_incapacidad | Cubierto |
| SAR | - | Aportación 2% SBC | calc/sar.rs | calcular_sar | sar_2pct | Cubierto |
| ISN | 32 estatales | Tasas por entidad | commands/fiscal.rs | listar_estados_isn | - | Parcial |
| CFDI | 4.0 / Nomina 1.2 | XML + catálogos XSD + mapeo semántico + identidades V1-V4 | commands/cfdi_cmd.rs | generar_cfdi | cfdi_* (19: mapeo 001/002/019/009, 4 rechazos XSD, parseo real, sensibilidad, totales calculados) | Estructura OK; timbrado real PENDIENTE (sin CSD ni PAC) |
| CFDI | TotalSueldos/TotalGravado/TotalExento calculados | Suma desde conceptos, excluye 022/023/025/039/044 | commands/cfdi_cmd.rs | totales_percepciones | cfdi_totales_percepciones_calculados | Cubierto |
| CFDI | Validación de totales vs conceptos | Rechaza total_percepciones distinto de suma | commands/cfdi_cmd.rs | generar_cfdi | cfdi_rechaza_totales_inconsistentes | Cubierto |
| CFDI | Nómina 1.2 / `c_TipoHoras` | Horas extra `TipoPercepcion="019"` con nodos `HorasExtra` (01 dobles, 02 triples, 03 simples) | commands/cfdi_cmd.rs | build_horas_extra, generar_cfdi | horas_extra.rs: cfdi_019_emite_horas_extra_y_valida_xsd, cfdi_horas_extra_rechaza_tipo_horas_invalido | Cubierto — XSD SAT validado |
| CFDI | Sello criptográfico | Pendiente (sin CSD) | - | - | - | Pendiente |
| CFDI | Timbrado con PAC | Pendiente (sin contrato PAC) | - | - | - | Pendiente |
| INEGI | UMA 2026 | Enero 113.14/3,439.46; desde feb 117.31/3,566.22 | migrations/0002_seed | (parámetro BD) | cargo test + tests/subsidio_2026.rs | Cubierto |
| Subsidio | Enero 15.59% UMA 2025 | 536.21 = 15.59% × 3,439.46 | migrations/0002_seed | cargar_param_vigente | subsidio_enero_2026_536_21 | Cubierto |
| Subsidio | Feb-dic 15.02% UMA 2026 | 535.65 = 15.02% × 3,566.22 | migrations/0002_seed | cargar_param_vigente | subsidio_febrero_2026_535_65 | Cubierto |
| Op | - | Importación CSV/Excel | - | - | - | Pendiente |
| Op | - | Backup/restore BD (art. 28 CFF, conservación) | commands/backup.rs | backup_db, restore_db, listar_backups | tests/backup.rs: backup_genera_archivo_valido, restore_reemplaza_bd, restore_rechaza_archivo_corrupto, backup_no_sobrescribe | Cubierto |
| Op | - | Log de auditoría (CFF art. 28, LSS art. 15) | migrations/0003_audit_log.sql, commands/auditoria.rs | listar_auditoria, configurar_usuario_auditoria | tests/auditoria.rs: trigger_insert_trabajador_crea_audit_log, trigger_update_trabajador_guarda_antes_y_despues, trigger_delete_trabajador_guarda_registro, audit_log_no_se_puede_actualizar, listar_auditoria_filtra_por_tabla | Cubierto |
| Op | - | Multi-empresa | - | - | - | Pendiente |
| Op | - | Recálculo retroactivo SBC | - | - | - | Pendiente |

## Conteo
- Cubierto: 28
- Estructura OK; timbrado real PENDIENTE: 1
- Parcial: 3
- Pendiente: 8
- Total filas: 40

## Confirmaciones solicitadas
- **PTU (LFT 117-131)**: Pendiente — no existe archivo ni función.
- **Ajuste anual (LISR 150)**: Pendiente — no existe archivo ni función.
- **Horas extra (LFT 66, 67-68)**: Cubierto — `calc/horas_extra.rs` + `migrations/0003_horas_extra.sql`, 9 dobles + 3 triples para 12 horas, CFDI 019 con `c_TipoHoras`, 9 tests dedicados en verde.
