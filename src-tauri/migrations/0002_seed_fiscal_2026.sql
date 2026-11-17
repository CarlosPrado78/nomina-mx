-- Seed fiscal 2026
-- UMA: enero 2026 usa valores 2025; desde 01-02-2026 aplica UMA 2026 (INEGI).
INSERT INTO tablas_fiscales (anio, tipo, vigencia_inicio, vigencia_fin, datos_json) VALUES
(2026, 'uma', '2026-01-01', '2026-01-31', '{"diaria":113.14,"mensual":3439.46,"anual":41273.52}'),
(2026, 'uma', '2026-02-01', '2026-12-31', '{"diaria":117.31,"mensual":3566.22,"anual":42794.64}'),
(2026, 'salario_minimo', '2026-01-01', '2026-12-31', '{"general":278.80,"frontera":419.88}');

-- Tarifa ISR mensual 2026 (Anexo 8 RMF) - muestra representativa
INSERT INTO tablas_fiscales (anio, tipo, vigencia_inicio, vigencia_fin, datos_json) VALUES
(2026, 'tarifa_isr_mensual', '2026-01-01', '2026-12-31', '[{"limite_inferior":0.01,"limite_superior":844.59,"cuota_fija":0.0,"tasa":1.92},{"limite_inferior":844.60,"limite_superior":7168.51,"cuota_fija":16.22,"tasa":6.40},{"limite_inferior":7168.52,"limite_superior":12598.02,"cuota_fija":420.95,"tasa":10.88},{"limite_inferior":12598.03,"limite_superior":14644.64,"cuota_fija":1011.68,"tasa":16.00},{"limite_inferior":14644.65,"limite_superior":17533.64,"cuota_fija":1339.14,"tasa":17.92},{"limite_inferior":17533.65,"limite_superior":35362.83,"cuota_fija":1856.84,"tasa":21.36},{"limite_inferior":35362.84,"limite_superior":55736.68,"cuota_fija":5665.16,"tasa":23.52},{"limite_inferior":55736.69,"limite_superior":106410.50,"cuota_fija":10457.09,"tasa":30.00},{"limite_inferior":106410.51,"limite_superior":141880.66,"cuota_fija":25659.23,"tasa":32.00},{"limite_inferior":141880.67,"limite_superior":425641.99,"cuota_fija":37009.69,"tasa":34.00},{"limite_inferior":425642.00,"limite_superior":999999999.0,"cuota_fija":133488.54,"tasa":35.00}]');

-- Tarifa ISR quincenal (aprox: mensual/2) - simplificada: mismos tramos escalados
INSERT INTO tablas_fiscales (anio, tipo, vigencia_inicio, vigencia_fin, datos_json) VALUES
(2026, 'tarifa_isr_quincenal', '2026-01-01', '2026-12-31', '[{"limite_inferior":0.01,"limite_superior":422.30,"cuota_fija":0.0,"tasa":1.92},{"limite_inferior":422.31,"limite_superior":3584.26,"cuota_fija":8.11,"tasa":6.40},{"limite_inferior":3584.27,"limite_superior":6299.01,"cuota_fija":210.70,"tasa":10.88},{"limite_inferior":6299.02,"limite_superior":7322.32,"cuota_fija":505.82,"tasa":16.00},{"limite_inferior":7322.33,"limite_superior":8766.82,"cuota_fija":669.66,"tasa":17.92},{"limite_inferior":8766.83,"limite_superior":17681.42,"cuota_fija":928.32,"tasa":21.36},{"limite_inferior":17681.43,"limite_superior":27868.34,"cuota_fija":2818.98,"tasa":23.52},{"limite_inferior":27868.35,"limite_superior":53205.25,"cuota_fija":5208.14,"tasa":30.00},{"limite_inferior":53205.26,"limite_superior":70940.33,"cuota_fija":12809.22,"tasa":32.00},{"limite_inferior":70940.34,"limite_superior":212820.99,"cuota_fija":18684.44,"tasa":34.00},{"limite_inferior":212821.00,"limite_superior":999999999.0,"cuota_fija":66920.33,"tasa":35.00}]');

-- Tarifa ISR semanal simplificada
INSERT INTO tablas_fiscales (anio, tipo, vigencia_inicio, vigencia_fin, datos_json) VALUES
(2026, 'tarifa_isr_semanal', '2026-01-01', '2026-12-31', '[{"limite_inferior":0.01,"limite_superior":193.96,"cuota_fija":0.0,"tasa":1.92},{"limite_inferior":193.97,"limite_superior":1681.98,"cuota_fija":3.72,"tasa":6.40},{"limite_inferior":1681.99,"limite_superior":2957.88,"cuota_fija":98.74,"tasa":10.88},{"limite_inferior":2957.89,"limite_superior":3437.04,"cuota_fija":237.09,"tasa":16.00},{"limite_inferior":3437.05,"limite_superior":4114.98,"cuota_fija":314.36,"tasa":17.92},{"limite_inferior":4114.99,"limite_superior":8299.08,"cuota_fija":435.64,"tasa":21.36},{"limite_inferior":8299.09,"limite_superior":13078.60,"cuota_fija":1317.01,"tasa":23.52},{"limite_inferior":13078.61,"limite_superior":25009.12,"cuota_fija":2443.83,"tasa":30.00},{"limite_inferior":25009.13,"limite_superior":33290.95,"cuota_fija":6023.00,"tasa":32.00},{"limite_inferior":33290.96,"limite_superior":100000.0,"cuota_fija":8673.18,"tasa":34.00},{"limite_inferior":100000.01,"limite_superior":999999999.0,"cuota_fija":31067.18,"tasa":35.00}]');

-- Subsidio al empleo 2026
INSERT INTO tablas_fiscales (anio, tipo, vigencia_inicio, vigencia_fin, datos_json) VALUES
(2026, 'subsidio_empleo', '2026-01-01', '2026-01-31', '{"maximo":536.21,"tope":11492.66}'),
(2026, 'subsidio_empleo', '2026-02-01', '2026-12-31', '{"maximo":535.65,"tope":11492.66}');

-- Cuotas IMSS (parametros generales)
INSERT INTO tablas_fiscales (anio, tipo, vigencia_inicio, vigencia_fin, datos_json) VALUES
(2026, 'cuotas_imss', '2026-01-01', '2026-12-31', '{"obrero":{"eym":0.625,"iv":0.625,"ceav":1.125,"ceav_excedente":0.40},"patronal":{"cuota_fija_pct_uma":20.40,"iv":1.75,"guarderia":1.00,"retiro":2.00,"cv_tope_pct":11.88}}');

-- Cuotas cesantia (referencia)
INSERT INTO tablas_fiscales (anio, tipo, vigencia_inicio, vigencia_fin, datos_json) VALUES
(2026, 'cuotas_cesantia', '2026-01-01', '2026-12-31', '{"niveles":"art168"}');

-- Tasas ISN por estado (32 entidades) - valores tipicos
INSERT INTO tablas_fiscales (anio, tipo, vigencia_inicio, vigencia_fin, datos_json) VALUES
(2026, 'tasa_isn_estados', '2026-01-01', '2026-12-31', '{"Aguascalientes":2.5,"Baja California":2.0,"Baja California Sur":2.5,"Campeche":2.0,"Coahuila":2.0,"Colima":2.0,"Chiapas":2.0,"Chihuahua":3.0,"Ciudad de Mexico":3.0,"Durango":2.0,"Guanajuato":2.3,"Guerrero":2.0,"Hidalgo":2.0,"Jalisco":2.0,"Mexico":3.0,"Michoacan":2.5,"Morelos":2.0,"Nayarit":2.0,"Nuevo Leon":3.0,"Oaxaca":2.0,"Puebla":2.5,"Queretaro":2.5,"Quintana Roo":3.0,"San Luis Potosi":2.5,"Sinaloa":2.5,"Sonora":2.5,"Tabasco":2.5,"Tamaulipas":2.5,"Tlaxcala":2.0,"Veracruz":2.0,"Yucatan":2.5,"Zacatecas":2.0}');