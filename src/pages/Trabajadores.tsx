import { useEffect, useState } from 'react';
import { useForm } from 'react-hook-form';
import { zodResolver } from '@hookform/resolvers/zod';
import { z } from 'zod';
import { api } from '../lib/tauri';
import { useAppStore } from '../lib/store';
import { validarCURP, validarRFC, validarNSS } from '../lib/validators';
import { Campo } from '../components/Campo';
import {
  ESTADOS_CIVILES,
  JORNADAS_LABORALES,
  PERIODICIDADES_PAGO,
  TIPOS_CONTRATO,
  TIPOS_DESCUENTO_INFONAVIT,
} from '../lib/catalogos';

const schema = z.object({
  empresa_id: z.coerce.number(),
  nombre: z.string().min(1),
  curp: z.string().refine(validarCURP, 'CURP invalida'),
  rfc: z.string().refine(validarRFC, 'RFC invalido'),
  nss: z.string().refine(validarNSS, 'NSS invalido'),
  num_empleado: z.string(),
  fecha_nacimiento: z.string(),
  estado_civil: z.string(),
  fecha_ingreso: z.string(),
  tipo_contrato: z.string(),
  departamento: z.string(),
  puesto: z.string(),
  salario_diario: z.coerce.number(),
  periodicidad_pago: z.string(),
  jornada: z.string(),
  horas_semanales: z.coerce.number(),
  infonavit_activo: z.boolean().optional(),
  infonavit_num_credito: z.string().optional(),
  infonavit_factor: z.coerce.number().optional(),
  infonavit_tipo_descuento: z.string().optional(),
});

const CLASE_CONTROL = 'border p-2 rounded w-full';
const VALORES_INICIALES = {
  empresa_id: 1,
  nombre: '', curp: '', rfc: '', nss: '', num_empleado: '',
  fecha_nacimiento: '', estado_civil: '', fecha_ingreso: '', tipo_contrato: '',
  departamento: '', puesto: '', salario_diario: 0,
  periodicidad_pago: 'mensual', jornada: 'diurna', horas_semanales: 48,
  infonavit_activo: false, infonavit_num_credito: '', infonavit_factor: 0,
  infonavit_tipo_descuento: 'CUOTA_FIJA_VSM',
};

export default function TrabajadoresPage() {
  const { register, handleSubmit, reset, watch } = useForm({
    resolver: zodResolver(schema),
    defaultValues: VALORES_INICIALES,
  });
  const [lista, setLista] = useState<any[]>([]);
  const [msg, setMsg] = useState('');
  const reloadKey = useAppStore((s) => s.reloadKey);
  const infonavitActivo = watch('infonavit_activo');

  const cargar = async () => {
    try { setLista(await api.listarTrabajadores(1)); } catch (e) { console.error(e); }
  };

  // C4: recarga cuando cambia la empresa activa.
  useEffect(() => { cargar(); }, [reloadKey]);

  const onSubmit = async (data: any) => {
    await api.guardarTrabajador(data);
    setMsg('Trabajador guardado');
    reset({ empresa_id: 1, infonavit_activo: false } as any);
    cargar();
  };

  return (
    <div>
      <h1 className="text-xl font-bold mb-1">Alta de trabajador</h1>
      <p className="text-sm text-gray-600 mb-4">
        Todas las fechas se capturan en formato AAAA-MM-DD (por ejemplo, 2026-01-15). Los importes
        son en pesos mexicanos y se escriben antes de impuestos.
      </p>
      <form onSubmit={handleSubmit(onSubmit)} className="grid grid-cols-1 md:grid-cols-3 gap-3 mb-6">
        <input type="hidden" {...register('empresa_id')} />

        <Campo id="trab-nombre" label="Nombre completo" hint="Nombre y apellidos del trabajador, sin abreviaturas.">
          <input {...register('nombre')} className={CLASE_CONTROL} placeholder="José Pérez Núñez" />
        </Campo>

        <Campo
          id="trab-curp"
          label="CURP"
          hint="18 caracteres. Si el trabajador es extranjero, usar la CURP genérica XEXX010101HNEXXXA4."
        >
          <input {...register('curp')} className={CLASE_CONTROL} placeholder="PENJ850101HDFRRN01" maxLength={18} />
        </Campo>

        <Campo id="trab-rfc" label="RFC" hint="13 caracteres para persona física.">
          <input {...register('rfc')} className={CLASE_CONTROL} placeholder="PENJ850101AB1" maxLength={13} />
        </Campo>

        <Campo id="trab-nss" label="NSS (Número de Seguridad Social)" hint="11 dígitos. Se obtiene del IMSS.">
          <input {...register('nss')} className={CLASE_CONTROL} placeholder="12345678901" maxLength={11} />
        </Campo>

        <Campo id="trab-num-empleado" label="Número de empleado" hint="Clave interna opcional de referencia; puede dejarla vacía.">
          <input {...register('num_empleado')} className={CLASE_CONTROL} placeholder="101" />
        </Campo>

        <Campo
          id="trab-fecha-nacimiento"
          label="Fecha de nacimiento"
          hint="Fecha en que nació el trabajador. Formato AAAA-MM-DD."
        >
          <input {...register('fecha_nacimiento')} type="date" className={CLASE_CONTROL} />
        </Campo>

        <Campo id="trab-estado-civil" label="Estado civil" hint="Estado civil del trabajador al momento del alta.">
          <select {...register('estado_civil')} className={CLASE_CONTROL}>
            <option value="">Seleccione…</option>
            {ESTADOS_CIVILES.map((e) => <option key={e.value} value={e.value}>{e.label}</option>)}
          </select>
        </Campo>

        <Campo
          id="trab-fecha-ingreso"
          label="Fecha de ingreso a la empresa"
          hint="Fecha en que el trabajador empezó a laborar en ESTA empresa. No es fecha de nacimiento ni fecha actual."
        >
          <input {...register('fecha_ingreso')} type="date" className={CLASE_CONTROL} />
        </Campo>

        <Campo id="trab-tipo-contrato" label="Tipo de contrato" hint="Modalidad de contratación; define indemnizaciones en finiquitos.">
          <select {...register('tipo_contrato')} className={CLASE_CONTROL}>
            <option value="">Seleccione…</option>
            {TIPOS_CONTRATO.map((t) => <option key={t.value} value={t.value}>{t.label}</option>)}
          </select>
        </Campo>

        <Campo id="trab-departamento" label="Departamento" hint="Área interna donde labora el trabajador. Ej. Producción.">
          <input {...register('departamento')} className={CLASE_CONTROL} placeholder="Producción" />
        </Campo>

        <Campo id="trab-puesto" label="Puesto" hint="Nombre del puesto o cargo. Ej. Operadora.">
          <input {...register('puesto')} className={CLASE_CONTROL} placeholder="Operadora" />
        </Campo>

        <Campo
          id="trab-salario-diario"
          label="Salario diario (pesos)"
          hint="Salario diario pactado, ANTES de impuestos. Si el salario es mensual, dividir entre 30."
        >
          <input {...register('salario_diario')} type="number" step="0.01" min="0" className={CLASE_CONTROL} placeholder="500.00" />
        </Campo>

        <Campo
          id="trab-periodicidad"
          label="Periodicidad de pago"
          hint="Frecuencia con la que se paga al trabajador."
        >
          <select {...register('periodicidad_pago')} className={CLASE_CONTROL}>
            {PERIODICIDADES_PAGO.map((p) => <option key={p.value} value={p.value}>{p.label}</option>)}
          </select>
        </Campo>

        <Campo id="trab-jornada" label="Jornada laboral" hint="Turno en que trabaja; afecta el cálculo de horas extra.">
          <select {...register('jornada')} className={CLASE_CONTROL}>
            {JORNADAS_LABORALES.map((j) => <option key={j.value} value={j.value}>{j.label}</option>)}
          </select>
        </Campo>

        <Campo id="trab-horas-semanales" label="Horas semanales" hint="Horas trabajadas por semana. Jornada completa = 48.">
          <input {...register('horas_semanales')} type="number" min="1" max="168" className={CLASE_CONTROL} placeholder="48" />
        </Campo>

        <Campo
          id="trab-infonavit-activo"
          label="INFONAVIT activo"
          hint="Activar solo si el trabajador tiene crédito INFONAVIT vigente. Marque la casilla para capturar los datos del crédito."
        >
          <input {...register('infonavit_activo')} type="checkbox" className="h-4 w-4" />
        </Campo>

        {infonavitActivo && (
          <>
            <Campo
              id="trab-infonavit-credito"
              label="Número de crédito INFONAVIT"
              hint="Número de contrato otorgado por el INFONAVIT. Ejemplo: 1234567890."
            >
              <input {...register('infonavit_num_credito')} className={CLASE_CONTROL} placeholder="1234567890" />
            </Campo>

            <Campo
              id="trab-infonavit-factor"
              label="Factor de descuento"
              hint="Valor del descuento: porcentaje, monto en pesos o VSM según el tipo elegido."
            >
              <input {...register('infonavit_factor')} type="number" step="0.0001" min="0" className={CLASE_CONTROL} placeholder="0.5" />
            </Campo>

            <Campo
              id="trab-infonavit-tipo"
              label="Tipo de descuento"
              hint="Porcentaje del salario, monto fijo en pesos o cuota fija en salarios mínimos (VSM)."
            >
              <select {...register('infonavit_tipo_descuento')} className={CLASE_CONTROL}>
                {TIPOS_DESCUENTO_INFONAVIT.map((t) => <option key={t.value} value={t.value}>{t.label}</option>)}
              </select>
            </Campo>
          </>
        )}

        <button type="submit" className="md:col-span-3 bg-blue-600 text-white p-2 rounded">Guardar trabajador</button>
        {msg && <span className="md:col-span-3 text-green-600">{msg}</span>}
      </form>

      <h2 className="text-lg font-bold mb-2">Trabajadores</h2>
      <table className="w-full border">
        <thead><tr className="bg-gray-100"><th>Nombre</th><th>RFC</th><th>Salario diario (pesos)</th></tr></thead>
        <tbody>
          {lista.map((t: any) => (
            <tr key={t.id} className="border-t">
              <td className="p-2">{t.nombre}</td>
              <td className="p-2">{t.rfc}</td>
              <td className="p-2">{t.salario_diario}</td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}
