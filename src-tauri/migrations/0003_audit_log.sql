-- C3 — Log de auditoría inmutable.
--
-- Fuentes:
--   CFF art. 28: conservación de documentación y registros contables.
--   LSS art. 15: registro de movimientos afiliatorios de los trabajadores.
--
-- Nota de versión: este archivo se registra como migración 4 en lib.rs porque
-- la versión 3 ya está ocupada por 0003_horas_extra.sql. Se conserva el nombre
-- solicitado en la definición del índice.
--
-- CURRENT_TIMESTAMP en SQLite devuelve la fecha/hora en UTC (ISO-8601).
-- El JSON de antes/después se construye con json_object() de SQLite (JSON1)
-- y se normaliza con serde_json en commands/auditoria.rs antes de entregarse.

CREATE TABLE IF NOT EXISTS audit_log (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  tabla TEXT NOT NULL,
  operacion TEXT NOT NULL CHECK (operacion IN ('INSERT', 'UPDATE', 'DELETE', 'RECALCULO')),
  registro_id INTEGER NOT NULL,
  antes_json TEXT,
  despues_json TEXT,
  usuario TEXT NOT NULL DEFAULT 'system',
  timestamp TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Sesión de auditoría: commands/auditoria.rs sincroniza aquí el usuario activo
-- (NOMINA_USUARIO o config.json; fallback 'system') y los triggers lo leen.
CREATE TABLE IF NOT EXISTS audit_sesion (
  id INTEGER PRIMARY KEY CHECK (id = 1),
  usuario TEXT NOT NULL DEFAULT 'system'
);
INSERT OR IGNORE INTO audit_sesion (id, usuario) VALUES (1, 'system');

-- Inmutabilidad: solo INSERT permitido en audit_log.
CREATE TRIGGER IF NOT EXISTS audit_log_no_update
BEFORE UPDATE ON audit_log
BEGIN
  SELECT RAISE(ABORT, 'audit_log es inmutable: UPDATE no permitido');
END;

CREATE TRIGGER IF NOT EXISTS audit_log_no_delete
BEFORE DELETE ON audit_log
BEGIN
  SELECT RAISE(ABORT, 'audit_log es inmutable: DELETE no permitido');
END;

-- ---------------------------------------------------------------------------
-- trabajadores
-- ---------------------------------------------------------------------------
CREATE TRIGGER IF NOT EXISTS ai_trabajadores
AFTER INSERT ON trabajadores
BEGIN
  INSERT INTO audit_log (tabla, operacion, registro_id, antes_json, despues_json, usuario)
  VALUES ('trabajadores', 'INSERT', NEW.id, NULL,
    json_object(
      'id', NEW.id, 'empresa_id', NEW.empresa_id, 'nombre', NEW.nombre,
      'curp', NEW.curp, 'rfc', NEW.rfc, 'nss', NEW.nss,
      'num_empleado', NEW.num_empleado, 'fecha_nacimiento', NEW.fecha_nacimiento,
      'estado_civil', NEW.estado_civil, 'fecha_ingreso', NEW.fecha_ingreso,
      'tipo_contrato', NEW.tipo_contrato, 'departamento', NEW.departamento,
      'puesto', NEW.puesto, 'salario_diario', NEW.salario_diario,
      'periodicidad_pago', NEW.periodicidad_pago, 'jornada', NEW.jornada,
      'horas_semanales', NEW.horas_semanales, 'infonavit_activo', NEW.infonavit_activo,
      'infonavit_num_credito', NEW.infonavit_num_credito, 'infonavit_factor', NEW.infonavit_factor,
      'infonavit_tipo_descuento', NEW.infonavit_tipo_descuento,
      'fecha_baja', NEW.fecha_baja, 'tipo_baja', NEW.tipo_baja
    ),
    (SELECT usuario FROM audit_sesion WHERE id = 1));
END;

CREATE TRIGGER IF NOT EXISTS au_trabajadores
AFTER UPDATE ON trabajadores
BEGIN
  INSERT INTO audit_log (tabla, operacion, registro_id, antes_json, despues_json, usuario)
  VALUES ('trabajadores', 'UPDATE', NEW.id,
    json_object(
      'id', OLD.id, 'empresa_id', OLD.empresa_id, 'nombre', OLD.nombre,
      'curp', OLD.curp, 'rfc', OLD.rfc, 'nss', OLD.nss,
      'num_empleado', OLD.num_empleado, 'fecha_nacimiento', OLD.fecha_nacimiento,
      'estado_civil', OLD.estado_civil, 'fecha_ingreso', OLD.fecha_ingreso,
      'tipo_contrato', OLD.tipo_contrato, 'departamento', OLD.departamento,
      'puesto', OLD.puesto, 'salario_diario', OLD.salario_diario,
      'periodicidad_pago', OLD.periodicidad_pago, 'jornada', OLD.jornada,
      'horas_semanales', OLD.horas_semanales, 'infonavit_activo', OLD.infonavit_activo,
      'infonavit_num_credito', OLD.infonavit_num_credito, 'infonavit_factor', OLD.infonavit_factor,
      'infonavit_tipo_descuento', OLD.infonavit_tipo_descuento,
      'fecha_baja', OLD.fecha_baja, 'tipo_baja', OLD.tipo_baja
    ),
    json_object(
      'id', NEW.id, 'empresa_id', NEW.empresa_id, 'nombre', NEW.nombre,
      'curp', NEW.curp, 'rfc', NEW.rfc, 'nss', NEW.nss,
      'num_empleado', NEW.num_empleado, 'fecha_nacimiento', NEW.fecha_nacimiento,
      'estado_civil', NEW.estado_civil, 'fecha_ingreso', NEW.fecha_ingreso,
      'tipo_contrato', NEW.tipo_contrato, 'departamento', NEW.departamento,
      'puesto', NEW.puesto, 'salario_diario', NEW.salario_diario,
      'periodicidad_pago', NEW.periodicidad_pago, 'jornada', NEW.jornada,
      'horas_semanales', NEW.horas_semanales, 'infonavit_activo', NEW.infonavit_activo,
      'infonavit_num_credito', NEW.infonavit_num_credito, 'infonavit_factor', NEW.infonavit_factor,
      'infonavit_tipo_descuento', NEW.infonavit_tipo_descuento,
      'fecha_baja', NEW.fecha_baja, 'tipo_baja', NEW.tipo_baja
    ),
    (SELECT usuario FROM audit_sesion WHERE id = 1));
END;

CREATE TRIGGER IF NOT EXISTS ad_trabajadores
AFTER DELETE ON trabajadores
BEGIN
  INSERT INTO audit_log (tabla, operacion, registro_id, antes_json, despues_json, usuario)
  VALUES ('trabajadores', 'DELETE', OLD.id,
    json_object(
      'id', OLD.id, 'empresa_id', OLD.empresa_id, 'nombre', OLD.nombre,
      'curp', OLD.curp, 'rfc', OLD.rfc, 'nss', OLD.nss,
      'num_empleado', OLD.num_empleado, 'fecha_nacimiento', OLD.fecha_nacimiento,
      'estado_civil', OLD.estado_civil, 'fecha_ingreso', OLD.fecha_ingreso,
      'tipo_contrato', OLD.tipo_contrato, 'departamento', OLD.departamento,
      'puesto', OLD.puesto, 'salario_diario', OLD.salario_diario,
      'periodicidad_pago', OLD.periodicidad_pago, 'jornada', OLD.jornada,
      'horas_semanales', OLD.horas_semanales, 'infonavit_activo', OLD.infonavit_activo,
      'infonavit_num_credito', OLD.infonavit_num_credito, 'infonavit_factor', OLD.infonavit_factor,
      'infonavit_tipo_descuento', OLD.infonavit_tipo_descuento,
      'fecha_baja', OLD.fecha_baja, 'tipo_baja', OLD.tipo_baja
    ),
    NULL,
    (SELECT usuario FROM audit_sesion WHERE id = 1));
END;

-- ---------------------------------------------------------------------------
-- periodos_nomina
-- ---------------------------------------------------------------------------
CREATE TRIGGER IF NOT EXISTS ai_periodos_nomina
AFTER INSERT ON periodos_nomina
BEGIN
  INSERT INTO audit_log (tabla, operacion, registro_id, antes_json, despues_json, usuario)
  VALUES ('periodos_nomina', 'INSERT', NEW.id, NULL,
    json_object('id', NEW.id, 'empresa_id', NEW.empresa_id, 'tipo', NEW.tipo,
      'fecha_inicio', NEW.fecha_inicio, 'fecha_fin', NEW.fecha_fin,
      'fecha_pago', NEW.fecha_pago, 'estado', NEW.estado, 'dias', NEW.dias),
    (SELECT usuario FROM audit_sesion WHERE id = 1));
END;

CREATE TRIGGER IF NOT EXISTS au_periodos_nomina
AFTER UPDATE ON periodos_nomina
BEGIN
  INSERT INTO audit_log (tabla, operacion, registro_id, antes_json, despues_json, usuario)
  VALUES ('periodos_nomina', 'UPDATE', NEW.id,
    json_object('id', OLD.id, 'empresa_id', OLD.empresa_id, 'tipo', OLD.tipo,
      'fecha_inicio', OLD.fecha_inicio, 'fecha_fin', OLD.fecha_fin,
      'fecha_pago', OLD.fecha_pago, 'estado', OLD.estado, 'dias', OLD.dias),
    json_object('id', NEW.id, 'empresa_id', NEW.empresa_id, 'tipo', NEW.tipo,
      'fecha_inicio', NEW.fecha_inicio, 'fecha_fin', NEW.fecha_fin,
      'fecha_pago', NEW.fecha_pago, 'estado', NEW.estado, 'dias', NEW.dias),
    (SELECT usuario FROM audit_sesion WHERE id = 1));
END;

CREATE TRIGGER IF NOT EXISTS ad_periodos_nomina
AFTER DELETE ON periodos_nomina
BEGIN
  INSERT INTO audit_log (tabla, operacion, registro_id, antes_json, despues_json, usuario)
  VALUES ('periodos_nomina', 'DELETE', OLD.id,
    json_object('id', OLD.id, 'empresa_id', OLD.empresa_id, 'tipo', OLD.tipo,
      'fecha_inicio', OLD.fecha_inicio, 'fecha_fin', OLD.fecha_fin,
      'fecha_pago', OLD.fecha_pago, 'estado', OLD.estado, 'dias', OLD.dias),
    NULL,
    (SELECT usuario FROM audit_sesion WHERE id = 1));
END;

-- ---------------------------------------------------------------------------
-- recibos
-- ---------------------------------------------------------------------------
CREATE TRIGGER IF NOT EXISTS ai_recibos
AFTER INSERT ON recibos
BEGIN
  INSERT INTO audit_log (tabla, operacion, registro_id, antes_json, despues_json, usuario)
  VALUES ('recibos', 'INSERT', NEW.id, NULL,
    json_object(
      'id', NEW.id, 'periodo_id', NEW.periodo_id, 'trabajador_id', NEW.trabajador_id,
      'sueldo_bruto', NEW.sueldo_bruto, 'isr', NEW.isr, 'imss_obrero', NEW.imss_obrero,
      'infonavit_desc', NEW.infonavit_desc, 'otras_deducciones', NEW.otras_deducciones,
      'total_percepciones', NEW.total_percepciones, 'total_deducciones', NEW.total_deducciones,
      'neto', NEW.neto, 'cuota_patronal_imss', NEW.cuota_patronal_imss,
      'aportacion_infonavit', NEW.aportacion_infonavit, 'aportacion_sar', NEW.aportacion_sar,
      'isn', NEW.isn, 'costo_total_patron', NEW.costo_total_patron,
      'xml_cfdi', NEW.xml_cfdi, 'uuid_cfdi', NEW.uuid_cfdi, 'fecha_timbrado', NEW.fecha_timbrado,
      'horas_extra_dobles', NEW.horas_extra_dobles, 'horas_extra_triples', NEW.horas_extra_triples,
      'horas_extra_importe', NEW.horas_extra_importe, 'horas_extra_exento', NEW.horas_extra_exento,
      'horas_extra_gravado', NEW.horas_extra_gravado
    ),
    (SELECT usuario FROM audit_sesion WHERE id = 1));
END;

CREATE TRIGGER IF NOT EXISTS au_recibos
AFTER UPDATE ON recibos
BEGIN
  INSERT INTO audit_log (tabla, operacion, registro_id, antes_json, despues_json, usuario)
  VALUES ('recibos', 'UPDATE', NEW.id,
    json_object(
      'id', OLD.id, 'periodo_id', OLD.periodo_id, 'trabajador_id', OLD.trabajador_id,
      'sueldo_bruto', OLD.sueldo_bruto, 'isr', OLD.isr, 'imss_obrero', OLD.imss_obrero,
      'infonavit_desc', OLD.infonavit_desc, 'otras_deducciones', OLD.otras_deducciones,
      'total_percepciones', OLD.total_percepciones, 'total_deducciones', OLD.total_deducciones,
      'neto', OLD.neto, 'cuota_patronal_imss', OLD.cuota_patronal_imss,
      'aportacion_infonavit', OLD.aportacion_infonavit, 'aportacion_sar', OLD.aportacion_sar,
      'isn', OLD.isn, 'costo_total_patron', OLD.costo_total_patron,
      'xml_cfdi', OLD.xml_cfdi, 'uuid_cfdi', OLD.uuid_cfdi, 'fecha_timbrado', OLD.fecha_timbrado,
      'horas_extra_dobles', OLD.horas_extra_dobles, 'horas_extra_triples', OLD.horas_extra_triples,
      'horas_extra_importe', OLD.horas_extra_importe, 'horas_extra_exento', OLD.horas_extra_exento,
      'horas_extra_gravado', OLD.horas_extra_gravado
    ),
    json_object(
      'id', NEW.id, 'periodo_id', NEW.periodo_id, 'trabajador_id', NEW.trabajador_id,
      'sueldo_bruto', NEW.sueldo_bruto, 'isr', NEW.isr, 'imss_obrero', NEW.imss_obrero,
      'infonavit_desc', NEW.infonavit_desc, 'otras_deducciones', NEW.otras_deducciones,
      'total_percepciones', NEW.total_percepciones, 'total_deducciones', NEW.total_deducciones,
      'neto', NEW.neto, 'cuota_patronal_imss', NEW.cuota_patronal_imss,
      'aportacion_infonavit', NEW.aportacion_infonavit, 'aportacion_sar', NEW.aportacion_sar,
      'isn', NEW.isn, 'costo_total_patron', NEW.costo_total_patron,
      'xml_cfdi', NEW.xml_cfdi, 'uuid_cfdi', NEW.uuid_cfdi, 'fecha_timbrado', NEW.fecha_timbrado,
      'horas_extra_dobles', NEW.horas_extra_dobles, 'horas_extra_triples', NEW.horas_extra_triples,
      'horas_extra_importe', NEW.horas_extra_importe, 'horas_extra_exento', NEW.horas_extra_exento,
      'horas_extra_gravado', NEW.horas_extra_gravado
    ),
    (SELECT usuario FROM audit_sesion WHERE id = 1));
END;

CREATE TRIGGER IF NOT EXISTS ad_recibos
AFTER DELETE ON recibos
BEGIN
  INSERT INTO audit_log (tabla, operacion, registro_id, antes_json, despues_json, usuario)
  VALUES ('recibos', 'DELETE', OLD.id,
    json_object(
      'id', OLD.id, 'periodo_id', OLD.periodo_id, 'trabajador_id', OLD.trabajador_id,
      'sueldo_bruto', OLD.sueldo_bruto, 'isr', OLD.isr, 'imss_obrero', OLD.imss_obrero,
      'infonavit_desc', OLD.infonavit_desc, 'otras_deducciones', OLD.otras_deducciones,
      'total_percepciones', OLD.total_percepciones, 'total_deducciones', OLD.total_deducciones,
      'neto', OLD.neto, 'cuota_patronal_imss', OLD.cuota_patronal_imss,
      'aportacion_infonavit', OLD.aportacion_infonavit, 'aportacion_sar', OLD.aportacion_sar,
      'isn', OLD.isn, 'costo_total_patron', OLD.costo_total_patron,
      'xml_cfdi', OLD.xml_cfdi, 'uuid_cfdi', OLD.uuid_cfdi, 'fecha_timbrado', OLD.fecha_timbrado,
      'horas_extra_dobles', OLD.horas_extra_dobles, 'horas_extra_triples', OLD.horas_extra_triples,
      'horas_extra_importe', OLD.horas_extra_importe, 'horas_extra_exento', OLD.horas_extra_exento,
      'horas_extra_gravado', OLD.horas_extra_gravado
    ),
    NULL,
    (SELECT usuario FROM audit_sesion WHERE id = 1));
END;

-- ---------------------------------------------------------------------------
-- incapacidades
-- ---------------------------------------------------------------------------
CREATE TRIGGER IF NOT EXISTS ai_incapacidades
AFTER INSERT ON incapacidades
BEGIN
  INSERT INTO audit_log (tabla, operacion, registro_id, antes_json, despues_json, usuario)
  VALUES ('incapacidades', 'INSERT', NEW.id, NULL,
    json_object('id', NEW.id, 'trabajador_id', NEW.trabajador_id, 'tipo', NEW.tipo,
      'fecha_inicio', NEW.fecha_inicio, 'fecha_fin', NEW.fecha_fin, 'dias', NEW.dias,
      'ramo_imss', NEW.ramo_imss, 'porcentaje_pago', NEW.porcentaje_pago),
    (SELECT usuario FROM audit_sesion WHERE id = 1));
END;

CREATE TRIGGER IF NOT EXISTS au_incapacidades
AFTER UPDATE ON incapacidades
BEGIN
  INSERT INTO audit_log (tabla, operacion, registro_id, antes_json, despues_json, usuario)
  VALUES ('incapacidades', 'UPDATE', NEW.id,
    json_object('id', OLD.id, 'trabajador_id', OLD.trabajador_id, 'tipo', OLD.tipo,
      'fecha_inicio', OLD.fecha_inicio, 'fecha_fin', OLD.fecha_fin, 'dias', OLD.dias,
      'ramo_imss', OLD.ramo_imss, 'porcentaje_pago', OLD.porcentaje_pago),
    json_object('id', NEW.id, 'trabajador_id', NEW.trabajador_id, 'tipo', NEW.tipo,
      'fecha_inicio', NEW.fecha_inicio, 'fecha_fin', NEW.fecha_fin, 'dias', NEW.dias,
      'ramo_imss', NEW.ramo_imss, 'porcentaje_pago', NEW.porcentaje_pago),
    (SELECT usuario FROM audit_sesion WHERE id = 1));
END;

CREATE TRIGGER IF NOT EXISTS ad_incapacidades
AFTER DELETE ON incapacidades
BEGIN
  INSERT INTO audit_log (tabla, operacion, registro_id, antes_json, despues_json, usuario)
  VALUES ('incapacidades', 'DELETE', OLD.id,
    json_object('id', OLD.id, 'trabajador_id', OLD.trabajador_id, 'tipo', OLD.tipo,
      'fecha_inicio', OLD.fecha_inicio, 'fecha_fin', OLD.fecha_fin, 'dias', OLD.dias,
      'ramo_imss', OLD.ramo_imss, 'porcentaje_pago', OLD.porcentaje_pago),
    NULL,
    (SELECT usuario FROM audit_sesion WHERE id = 1));
END;

-- ---------------------------------------------------------------------------
-- vacaciones
-- ---------------------------------------------------------------------------
CREATE TRIGGER IF NOT EXISTS ai_vacaciones
AFTER INSERT ON vacaciones
BEGIN
  INSERT INTO audit_log (tabla, operacion, registro_id, antes_json, despues_json, usuario)
  VALUES ('vacaciones', 'INSERT', NEW.id, NULL,
    json_object('id', NEW.id, 'trabajador_id', NEW.trabajador_id,
      'fecha_inicio', NEW.fecha_inicio, 'fecha_fin', NEW.fecha_fin, 'dias', NEW.dias,
      'prima_pagada', NEW.prima_pagada, 'periodo_aniversario', NEW.periodo_aniversario),
    (SELECT usuario FROM audit_sesion WHERE id = 1));
END;

CREATE TRIGGER IF NOT EXISTS au_vacaciones
AFTER UPDATE ON vacaciones
BEGIN
  INSERT INTO audit_log (tabla, operacion, registro_id, antes_json, despues_json, usuario)
  VALUES ('vacaciones', 'UPDATE', NEW.id,
    json_object('id', OLD.id, 'trabajador_id', OLD.trabajador_id,
      'fecha_inicio', OLD.fecha_inicio, 'fecha_fin', OLD.fecha_fin, 'dias', OLD.dias,
      'prima_pagada', OLD.prima_pagada, 'periodo_aniversario', OLD.periodo_aniversario),
    json_object('id', NEW.id, 'trabajador_id', NEW.trabajador_id,
      'fecha_inicio', NEW.fecha_inicio, 'fecha_fin', NEW.fecha_fin, 'dias', NEW.dias,
      'prima_pagada', NEW.prima_pagada, 'periodo_aniversario', NEW.periodo_aniversario),
    (SELECT usuario FROM audit_sesion WHERE id = 1));
END;

CREATE TRIGGER IF NOT EXISTS ad_vacaciones
AFTER DELETE ON vacaciones
BEGIN
  INSERT INTO audit_log (tabla, operacion, registro_id, antes_json, despues_json, usuario)
  VALUES ('vacaciones', 'DELETE', OLD.id,
    json_object('id', OLD.id, 'trabajador_id', OLD.trabajador_id,
      'fecha_inicio', OLD.fecha_inicio, 'fecha_fin', OLD.fecha_fin, 'dias', OLD.dias,
      'prima_pagada', OLD.prima_pagada, 'periodo_aniversario', OLD.periodo_aniversario),
    NULL,
    (SELECT usuario FROM audit_sesion WHERE id = 1));
END;
