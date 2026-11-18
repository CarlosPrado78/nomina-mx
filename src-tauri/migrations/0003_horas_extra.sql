-- Bloque B: parametros de tiempo extraordinario con vigencia.
-- Fuentes:
--   LFT art. 66 y 68 (texto vigente desde 01-05-2026, DOF 01-05-2026):
--     "un cien por ciento mas" = 2x (dobles); "doscientos por ciento mas" = 3x (triples).
--   Cuarto Transitorio del DOF 01-05-2026: en 2026 el tope de dobles sigue siendo 9 horas
--     semanales; a partir de 2030 sera 12. Las horas que excedan el tope se pagan al triple
--     (maximo 4 horas semanales, art. 68) sin que eso legalice la prolongacion.
--   LFT art. 61: jornada diurna 8 h, nocturna 7 h, mixta 7.5 h (base del valor por hora).
--   LISR art. 93 fracc. I: 50% exento (100% para trabajador de salario minimo) dentro del
--     limite laboral; tope de 5 veces la unidad de referencia por semana de servicios.
--     Se usa UMA por la desindexacion del salario minimo (art. 26 apartado C CPEUM).
INSERT INTO tablas_fiscales (anio, tipo, vigencia_inicio, vigencia_fin, datos_json) VALUES
(2026, 'horas_extra', '2026-01-01', '2026-12-31', '{"factor_dobles":2.0,"factor_triples":3.0,"limite_dobles_horas":9,"limite_triples_horas":4,"porcentaje_exento":0.5,"tope_exento_umas_semana":5,"horas_jornada_diurna":8,"horas_jornada_nocturna":7,"horas_jornada_mixta":7.5}');

-- Persistencia del desglose en el recibo.
ALTER TABLE recibos ADD COLUMN horas_extra_dobles REAL DEFAULT 0;
ALTER TABLE recibos ADD COLUMN horas_extra_triples REAL DEFAULT 0;
ALTER TABLE recibos ADD COLUMN horas_extra_importe REAL DEFAULT 0;
ALTER TABLE recibos ADD COLUMN horas_extra_exento REAL DEFAULT 0;
ALTER TABLE recibos ADD COLUMN horas_extra_gravado REAL DEFAULT 0;
