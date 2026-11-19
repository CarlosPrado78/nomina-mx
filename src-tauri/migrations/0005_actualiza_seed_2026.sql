-- Migración 6: corrige el seed fiscal 2026 en bases ya instaladas.
--
-- La migración 2 original tenía una sola fila de UMA (113.14 todo el año) y
-- una sola fila de subsidio (535.65 todo el año). El seed correcto distingue
-- enero de febrero-diciembre. Como una migración ya aplicada no se debe
-- editar, los cambios se aplican aquí como versión nueva.
--
-- Es idempotente: borrar y volver a insertar evita duplicados si la migración
-- se reintenta después de una interrupción.

DELETE FROM tablas_fiscales WHERE anio = 2026 AND tipo = 'uma';
DELETE FROM tablas_fiscales WHERE anio = 2026 AND tipo = 'subsidio_empleo';

INSERT INTO tablas_fiscales (anio, tipo, vigencia_inicio, vigencia_fin, datos_json) VALUES
(2026, 'uma', '2026-01-01', '2026-01-31', '{"diaria":113.14,"mensual":3439.46,"anual":41273.52}'),
(2026, 'uma', '2026-02-01', '2026-12-31', '{"diaria":117.31,"mensual":3566.22,"anual":42794.64}');

INSERT INTO tablas_fiscales (anio, tipo, vigencia_inicio, vigencia_fin, datos_json) VALUES
(2026, 'subsidio_empleo', '2026-01-01', '2026-01-31', '{"maximo":536.21,"tope":11492.66}'),
(2026, 'subsidio_empleo', '2026-02-01', '2026-12-31', '{"maximo":535.65,"tope":11492.66}');
