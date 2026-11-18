import { invoke } from '@tauri-apps/api/core';

/** C2 — Resultado de una importación CSV (errores por fila, no por archivo). */
export interface ImportResult {
  insertados: number;
  errores: Array<{
    fila: number;
    columna: string;
    mensaje: string;
  }>;
}

/** C5 — Diferencia detectada al recalcular un recibo. */
export interface Diferencia {
  trabajador_id: number;
  nombre: string;
  percepciones_antes: number;
  percepciones_despues: number;
  deducciones_antes: number;
  deducciones_despues: number;
  neto_antes: number;
  neto_despues: number;
  delta_neto: number;
}

/** C5 — Resultado de un recálculo retroactivo. */
export interface ResultadoRecalculo {
  recibos_recalculados: number;
  diferencias: Diferencia[];
}

export const api = {
  guardarEmpresa: (empresa: any) => invoke('guardar_empresa', { empresa }),
  obtenerEmpresa: () => invoke<any>('obtener_empresa'),
  listarEmpresas: () => invoke<any[]>('listar_empresas'),
  cambiarEmpresaActiva: (id: number) => invoke<void>('cambiar_empresa_activa', { id }),
  crearEmpresa: (nueva: any) => invoke<any>('crear_empresa', { nueva }),
  guardarTrabajador: (trabajador: any) => invoke('guardar_trabajador', { t: trabajador }),
  listarTrabajadores: (empresaId: number) => invoke<any[]>('listar_trabajadores', { empresaId }),
  obtenerTrabajador: (id: number) => invoke<any>('obtener_trabajador', { id }),
  importarTrabajadoresCsv: (ruta: string) =>
    invoke<ImportResult>('importar_trabajadores_csv', { ruta }),
  recalcularPeriodo: (periodoId: number, motivo: string) =>
    invoke<ResultadoRecalculo>('recalcular_periodo', { periodoId, motivo }),
  recalcularTrabajadorPeriodo: (trabajadorId: number, periodoId: number) =>
    invoke<ResultadoRecalculo>('recalcular_trabajador_periodo', { trabajadorId, periodoId }),
  calcularRecibo: (trabajadorId: number, empresaId: number, dias: number, periodicidad: string, fechaPago: string, horasExtra?: any) =>
    invoke<any>('calcular_recibo', { trabajadorId, empresaId, dias, periodicidad, fechaPago, horasExtra }),
  guardarPeriodo: (periodo: any) => invoke('guardar_periodo', { periodo }),
  listarPeriodos: (empresaId: number) => invoke<any[]>('listar_periodos', { empresaId }),
  guardarRecibo: (periodoId: number, recibo: any) => invoke('guardar_recibo', { periodoId, r: recibo }),
  calcularFiniquito: (params: any) => invoke<any>('calcular_finiquito', params),
  calcularLiquidacion: (params: any) => invoke<any>('calcular_liquidacion', params),
  diasVacaciones: (anios: number) => invoke<number>('dias_vacaciones', { anios }),
  obtenerParametros: (anio: number, tipo: string) => invoke<any>('obtener_parametros', { anio, tipo }),
  guardarParametro: (param: any) => invoke('guardar_parametro', { param }),
  listarEstadosIsn: () => invoke<any[]>('listar_estados_isn'),
  generarCfdi: (req: any) => invoke<any>('generar_cfdi', { req }),
  reporteCuotasImss: (empresaId: number) => invoke<any[]>('reporte_cuotas_imss', { empresaId }),
  listarAuditoria: (filtros: any) => invoke<any[]>('listar_auditoria', { filtros }),
  configurarUsuarioAuditoria: () => invoke<string>('configurar_usuario_auditoria'),
};