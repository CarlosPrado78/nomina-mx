-- C4 — Multi-empresa con selección persistida.
--
-- Fuentes: LISR art. 79 (requisitos de deducción) y CFF art. 27
-- (contabilidad separada por establecimiento).
--
-- Nota de versión: se registra como migración 5 en lib.rs (1 init,
-- 2 seed fiscal, 3 horas extra, 4 auditoría).
--
-- Todos los registros existentes se asignan a Empresa Default (id = 1).
-- La empresa activa se guarda en config(clave='empresa_activa', valor=TEXT).

CREATE TABLE IF NOT EXISTS config (
  clave TEXT PRIMARY KEY,
  valor TEXT NOT NULL
);

-- RFC único por empresa, enforzado en la propia BD.
CREATE UNIQUE INDEX IF NOT EXISTS ux_empresas_rfc ON empresas(rfc);

-- Empresa Default: IF NOT EXISTS por id, OR IGNORE para no duplicar RFC.
INSERT OR IGNORE INTO empresas (
  id, rfc, razon_social, regimen_fiscal, cp, estado_isn,
  tasa_isn, prima_riesgo_trabajo, registro_patronal_imss
) VALUES (1, 'XAXX010101000', 'Empresa Default', '601', '00000',
          'CIUDAD DE MEXICO', 3.0, 0.5, '');

-- Si el usuario ya tenía una selección, DO NOTHING la conserva.
INSERT INTO config (clave, valor)
VALUES ('empresa_activa',
        COALESCE((SELECT id FROM empresas WHERE id = 1),
                 (SELECT MIN(id) FROM empresas)))
ON CONFLICT(clave) DO NOTHING;

-- Las tablas trabajadores y periodos_nomina ya tenían empresa_id NOT NULL.
ALTER TABLE recibos        ADD COLUMN empresa_id INTEGER NOT NULL DEFAULT 1;
ALTER TABLE incapacidades  ADD COLUMN empresa_id INTEGER NOT NULL DEFAULT 1;
ALTER TABLE vacaciones     ADD COLUMN empresa_id INTEGER NOT NULL DEFAULT 1;

-- Regla C4: TODOS los registros existentes pasan a la Empresa Default (id=1).
-- En las tres tablas nuevas el DEFAULT ya lo garantiza; se repite de forma
-- explícita para las cinco tablas y hacer verificable la regla de negocio.
UPDATE trabajadores     SET empresa_id = 1;
UPDATE periodos_nomina  SET empresa_id = 1;
UPDATE recibos          SET empresa_id = 1;
UPDATE incapacidades    SET empresa_id = 1;
UPDATE vacaciones       SET empresa_id = 1;

-- ---------------------------------------------------------------------------
-- Los snapshots de auditoría deben incluir el nuevo empresa_id.
-- Se recrean solo los triggers de las tres tablas modificadas.
-- ---------------------------------------------------------------------------
DROP TRIGGER IF EXISTS ai_recibos;
CREATE TRIGGER ai_recibos
AFTER INSERT ON recibos
BEGIN
  INSERT INTO audit_log (tabla, operacion, registro_id, antes_json, despues_json, usuario)
  VALUES ('recibos', 'INSERT', NEW.id, NULL,
    json_object(
      'id', NEW.id, 'empresa_id', NEW.empresa_id,
      'periodo_id', NEW.periodo_id, 'trabajador_id', NEW.trabajador_id,
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

DROP TRIGGER IF EXISTS au_recibos;
CREATE TRIGGER au_recibos
AFTER UPDATE ON recibos
BEGIN
  INSERT INTO audit_log (tabla, operacion, registro_id, antes_json, despues_json, usuario)
  VALUES ('recibos', 'UPDATE', NEW.id,
    json_object(
      'id', OLD.id, 'empresa_id', OLD.empresa_id,
      'periodo_id', OLD.periodo_id, 'trabajador_id', OLD.trabajador_id,
      'sueldo_bruto', OLD.sueldo_bruto, 'isr', OLD.isr, 'imss_obrero', OLD.imss_obrero,
      'neto', OLD.neto, 'uuid_cfdi', OLD.uuid_cfdi
    ),
    json_object(
      'id', NEW.id, 'empresa_id', NEW.empresa_id,
      'periodo_id', NEW.periodo_id, 'trabajador_id', NEW.trabajador_id,
      'sueldo_bruto', NEW.sueldo_bruto, 'isr', NEW.isr, 'imss_obrero', NEW.imss_obrero,
      'neto', NEW.neto, 'uuid_cfdi', NEW.uuid_cfdi
    ),
    (SELECT usuario FROM audit_sesion WHERE id = 1));
END;

DROP TRIGGER IF EXISTS ad_recibos;
CREATE TRIGGER ad_recibos
AFTER DELETE ON recibos
BEGIN
  INSERT INTO audit_log (tabla, operacion, registro_id, antes_json, despues_json, usuario)
  VALUES ('recibos', 'DELETE', OLD.id,
    json_object(
      'id', OLD.id, 'empresa_id', OLD.empresa_id,
      'periodo_id', OLD.periodo_id, 'trabajador_id', OLD.trabajador_id,
      'neto', OLD.neto, 'uuid_cfdi', OLD.uuid_cfdi
    ),
    NULL,
    (SELECT usuario FROM audit_sesion WHERE id = 1));
END;

DROP TRIGGER IF EXISTS ai_incapacidades;
CREATE TRIGGER ai_incapacidades
AFTER INSERT ON incapacidades
BEGIN
  INSERT INTO audit_log (tabla, operacion, registro_id, antes_json, despues_json, usuario)
  VALUES ('incapacidades', 'INSERT', NEW.id, NULL,
    json_object('id', NEW.id, 'empresa_id', NEW.empresa_id,
      'trabajador_id', NEW.trabajador_id, 'tipo', NEW.tipo,
      'fecha_inicio', NEW.fecha_inicio, 'fecha_fin', NEW.fecha_fin, 'dias', NEW.dias,
      'ramo_imss', NEW.ramo_imss, 'porcentaje_pago', NEW.porcentaje_pago),
    (SELECT usuario FROM audit_sesion WHERE id = 1));
END;

DROP TRIGGER IF EXISTS au_incapacidades;
CREATE TRIGGER au_incapacidades
AFTER UPDATE ON incapacidades
BEGIN
  INSERT INTO audit_log (tabla, operacion, registro_id, antes_json, despues_json, usuario)
  VALUES ('incapacidades', 'UPDATE', NEW.id,
    json_object('id', OLD.id, 'empresa_id', OLD.empresa_id,
      'trabajador_id', OLD.trabajador_id, 'tipo', OLD.tipo,
      'fecha_inicio', OLD.fecha_inicio, 'fecha_fin', OLD.fecha_fin, 'dias', OLD.dias,
      'ramo_imss', OLD.ramo_imss, 'porcentaje_pago', OLD.porcentaje_pago),
    json_object('id', NEW.id, 'empresa_id', NEW.empresa_id,
      'trabajador_id', NEW.trabajador_id, 'tipo', NEW.tipo,
      'fecha_inicio', NEW.fecha_inicio, 'fecha_fin', NEW.fecha_fin, 'dias', NEW.dias,
      'ramo_imss', NEW.ramo_imss, 'porcentaje_pago', NEW.porcentaje_pago),
    (SELECT usuario FROM audit_sesion WHERE id = 1));
END;

DROP TRIGGER IF EXISTS ad_incapacidades;
CREATE TRIGGER ad_incapacidades
AFTER DELETE ON incapacidades
BEGIN
  INSERT INTO audit_log (tabla, operacion, registro_id, antes_json, despues_json, usuario)
  VALUES ('incapacidades', 'DELETE', OLD.id,
    json_object('id', OLD.id, 'empresa_id', OLD.empresa_id,
      'trabajador_id', OLD.trabajador_id, 'tipo', OLD.tipo,
      'fecha_inicio', OLD.fecha_inicio, 'fecha_fin', OLD.fecha_fin, 'dias', OLD.dias,
      'ramo_imss', OLD.ramo_imss, 'porcentaje_pago', OLD.porcentaje_pago),
    NULL,
    (SELECT usuario FROM audit_sesion WHERE id = 1));
END;

DROP TRIGGER IF EXISTS ai_vacaciones;
CREATE TRIGGER ai_vacaciones
AFTER INSERT ON vacaciones
BEGIN
  INSERT INTO audit_log (tabla, operacion, registro_id, antes_json, despues_json, usuario)
  VALUES ('vacaciones', 'INSERT', NEW.id, NULL,
    json_object('id', NEW.id, 'empresa_id', NEW.empresa_id,
      'trabajador_id', NEW.trabajador_id,
      'fecha_inicio', NEW.fecha_inicio, 'fecha_fin', NEW.fecha_fin, 'dias', NEW.dias,
      'prima_pagada', NEW.prima_pagada, 'periodo_aniversario', NEW.periodo_aniversario),
    (SELECT usuario FROM audit_sesion WHERE id = 1));
END;

DROP TRIGGER IF EXISTS au_vacaciones;
CREATE TRIGGER au_vacaciones
AFTER UPDATE ON vacaciones
BEGIN
  INSERT INTO audit_log (tabla, operacion, registro_id, antes_json, despues_json, usuario)
  VALUES ('vacaciones', 'UPDATE', NEW.id,
    json_object('id', OLD.id, 'empresa_id', OLD.empresa_id,
      'trabajador_id', OLD.trabajador_id,
      'fecha_inicio', OLD.fecha_inicio, 'fecha_fin', OLD.fecha_fin, 'dias', OLD.dias,
      'prima_pagada', OLD.prima_pagada, 'periodo_aniversario', OLD.periodo_aniversario),
    json_object('id', NEW.id, 'empresa_id', NEW.empresa_id,
      'trabajador_id', NEW.trabajador_id,
      'fecha_inicio', NEW.fecha_inicio, 'fecha_fin', NEW.fecha_fin, 'dias', NEW.dias,
      'prima_pagada', NEW.prima_pagada, 'periodo_aniversario', NEW.periodo_aniversario),
    (SELECT usuario FROM audit_sesion WHERE id = 1));
END;

DROP TRIGGER IF EXISTS ad_vacaciones;
CREATE TRIGGER ad_vacaciones
AFTER DELETE ON vacaciones
BEGIN
  INSERT INTO audit_log (tabla, operacion, registro_id, antes_json, despues_json, usuario)
  VALUES ('vacaciones', 'DELETE', OLD.id,
    json_object('id', OLD.id, 'empresa_id', OLD.empresa_id,
      'trabajador_id', OLD.trabajador_id,
      'fecha_inicio', OLD.fecha_inicio, 'fecha_fin', OLD.fecha_fin, 'dias', OLD.dias,
      'prima_pagada', OLD.prima_pagada, 'periodo_aniversario', OLD.periodo_aniversario),
    NULL,
    (SELECT usuario FROM audit_sesion WHERE id = 1));
END;
