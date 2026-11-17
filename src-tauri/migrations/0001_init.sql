CREATE TABLE IF NOT EXISTS empresas (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  rfc TEXT NOT NULL,
  razon_social TEXT NOT NULL,
  regimen_fiscal TEXT,
  cp TEXT,
  estado_isn TEXT,
  tasa_isn REAL DEFAULT 0,
  prima_riesgo_trabajo REAL DEFAULT 0.5,
  registro_patronal_imss TEXT
);

CREATE TABLE IF NOT EXISTS trabajadores (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  empresa_id INTEGER NOT NULL,
  nombre TEXT NOT NULL,
  curp TEXT,
  rfc TEXT,
  nss TEXT,
  num_empleado TEXT,
  fecha_nacimiento TEXT,
  estado_civil TEXT,
  fecha_ingreso TEXT,
  tipo_contrato TEXT,
  departamento TEXT,
  puesto TEXT,
  salario_diario REAL NOT NULL,
  periodicidad_pago TEXT,
  jornada TEXT,
  horas_semanales REAL,
  infonavit_activo INTEGER DEFAULT 0,
  infonavit_num_credito TEXT,
  infonavit_factor REAL DEFAULT 0,
  infonavit_tipo_descuento TEXT,
  fecha_baja TEXT,
  tipo_baja TEXT,
  FOREIGN KEY(empresa_id) REFERENCES empresas(id)
);

CREATE TABLE IF NOT EXISTS prestaciones (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  trabajador_id INTEGER NOT NULL,
  tipo TEXT,
  valor REAL,
  gravado INTEGER DEFAULT 1,
  integra_sbc INTEGER DEFAULT 1,
  FOREIGN KEY(trabajador_id) REFERENCES trabajadores(id)
);

CREATE TABLE IF NOT EXISTS periodos_nomina (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  empresa_id INTEGER NOT NULL,
  tipo TEXT,
  fecha_inicio TEXT,
  fecha_fin TEXT,
  fecha_pago TEXT,
  estado TEXT DEFAULT 'borrador',
  dias INTEGER,
  FOREIGN KEY(empresa_id) REFERENCES empresas(id)
);

CREATE TABLE IF NOT EXISTS recibos (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  periodo_id INTEGER NOT NULL,
  trabajador_id INTEGER NOT NULL,
  sueldo_bruto REAL,
  isr REAL,
  imss_obrero REAL,
  infonavit_desc REAL,
  otras_deducciones REAL,
  total_percepciones REAL,
  total_deducciones REAL,
  neto REAL,
  cuota_patronal_imss REAL,
  aportacion_infonavit REAL,
  aportacion_sar REAL,
  isn REAL,
  costo_total_patron REAL,
  xml_cfdi TEXT,
  uuid_cfdi TEXT,
  fecha_timbrado TEXT,
  FOREIGN KEY(periodo_id) REFERENCES periodos_nomina(id),
  FOREIGN KEY(trabajador_id) REFERENCES trabajadores(id)
);

CREATE TABLE IF NOT EXISTS acumulados_anuales (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  trabajador_id INTEGER NOT NULL,
  anio INTEGER,
  total_gravado REAL,
  total_exento REAL,
  isr_retenido REAL,
  subsidio_aplicado REAL,
  FOREIGN KEY(trabajador_id) REFERENCES trabajadores(id)
);

CREATE TABLE IF NOT EXISTS tablas_fiscales (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  anio INTEGER,
  tipo TEXT,
  vigencia_inicio TEXT,
  vigencia_fin TEXT,
  datos_json TEXT
);

CREATE TABLE IF NOT EXISTS incapacidades (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  trabajador_id INTEGER NOT NULL,
  tipo TEXT,
  fecha_inicio TEXT,
  fecha_fin TEXT,
  dias INTEGER,
  ramo_imss TEXT,
  porcentaje_pago REAL,
  FOREIGN KEY(trabajador_id) REFERENCES trabajadores(id)
);

CREATE TABLE IF NOT EXISTS vacaciones (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  trabajador_id INTEGER NOT NULL,
  fecha_inicio TEXT,
  fecha_fin TEXT,
  dias INTEGER,
  prima_pagada REAL,
  periodo_aniversario TEXT,
  FOREIGN KEY(trabajador_id) REFERENCES trabajadores(id)
);