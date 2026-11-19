/** Catálogos compartidos por los formularios. Los values son los que espera el backend. */

export const ESTADOS_REPUBLICA = [
  'AGUASCALIENTES', 'BAJA CALIFORNIA', 'BAJA CALIFORNIA SUR', 'CAMPECHE',
  'CHIAPAS', 'CHIHUAHUA', 'CIUDAD DE MEXICO', 'COAHUILA DE ZARAGOZA',
  'COLIMA', 'DURANGO', 'ESTADO DE MEXICO', 'GUANAJUATO', 'GUERRERO',
  'HIDALGO', 'JALISCO', 'MICHOACAN DE OCAMPO', 'MORELOS', 'NAYARIT',
  'NUEVO LEON', 'OAXACA', 'PUEBLA', 'QUERETARO', 'QUINTANA ROO',
  'SAN LUIS POTOSI', 'SINALOA', 'SONORA', 'TABASCO', 'TAMAULIPAS',
  'TLAXCALA', 'VERACRUZ DE IGNACIO DE LA LLAVE', 'YUCATAN', 'ZACATECAS',
];

export const REGIMENES_FISCALES = [
  { value: '601', label: '601 — General de Ley Personas Morales' },
  { value: '605', label: '605 — Sueldos y Salarios e Ingresos Asimilados' },
  { value: '612', label: '612 — Personas Físicas con Actividades Empresariales' },
  { value: '626', label: '626 — Remuneraciones por Servicios Personales' },
];

export const ESTADOS_CIVILES = [
  { value: 'soltero', label: 'Soltero/a' },
  { value: 'casado', label: 'Casado/a' },
  { value: 'union_libre', label: 'Unión libre' },
  { value: 'divorciado', label: 'Divorciado/a' },
  { value: 'viudo', label: 'Viudo/a' },
];

export const TIPOS_CONTRATO = [
  { value: 'indefinido', label: 'Indefinido' },
  { value: 'temporal', label: 'Temporal' },
  { value: 'obra_determinada', label: 'Obra determinada' },
];

export const PERIODICIDADES_PAGO = [
  { value: 'diaria', label: 'Diaria' },
  { value: 'semanal', label: 'Semanal' },
  { value: 'quincenal', label: 'Quincenal' },
  { value: 'mensual', label: 'Mensual' },
  { value: 'bimestral', label: 'Bimestral' },
  { value: 'trimestral', label: 'Trimestral' },
  { value: 'semestral', label: 'Semestral' },
  { value: 'anual', label: 'Anual' },
];

export const JORNADAS_LABORALES = [
  { value: 'diurna', label: 'Diurna' },
  { value: 'nocturna', label: 'Nocturna' },
  { value: 'mixta', label: 'Mixta' },
  { value: 'reducida', label: 'Reducida' },
];

export const TIPOS_DESCUENTO_INFONAVIT = [
  { value: 'CUOTA_FIJA_VSM', label: 'Cuota fija en VSM' },
  { value: 'PORCENTAJE', label: 'Porcentaje del salario' },
  { value: 'MONTO', label: 'Monto fijo en pesos' },
];

export const TIPOS_PERIODO = [
  { value: 'semanal', label: 'Semanal', dias: 7 },
  { value: 'quincenal', label: 'Quincenal', dias: 15 },
  { value: 'mensual', label: 'Mensual', dias: 30 },
  { value: 'decenal', label: 'Decenal', dias: 10 },
];

export const TIPOS_SEPARACION = [
  { value: 'renuncia', label: 'Renuncia' },
  { value: 'despido_justificado', label: 'Despido justificado' },
  { value: 'despido_injustificado', label: 'Despido injustificado' },
  { value: 'terminacion_contrato', label: 'Terminación de contrato' },
  { value: 'muerte', label: 'Muerte' },
  { value: 'incapacidad', label: 'Incapacidad' },
];
