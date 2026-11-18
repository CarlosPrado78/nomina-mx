import { invoke } from '@tauri-apps/api/core';

export const api = {
  guardarEmpresa: (empresa: any) => invoke('guardar_empresa', { empresa }),
  obtenerEmpresa: () => invoke<any>('obtener_empresa'),
  guardarTrabajador: (trabajador: any) => invoke('guardar_trabajador', { t: trabajador }),
  listarTrabajadores: (empresaId: number) => invoke<any[]>('listar_trabajadores', { empresaId }),
  obtenerTrabajador: (id: number) => invoke<any>('obtener_trabajador', { id }),
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
};