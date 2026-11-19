import { useEffect, useState } from 'react';
import { useForm } from 'react-hook-form';
import { zodResolver } from '@hookform/resolvers/zod';
import { z } from 'zod';
import { api } from '../lib/tauri';
import { useAppStore } from '../lib/store';
import { Campo } from '../components/Campo';
import { ESTADOS_REPUBLICA, REGIMENES_FISCALES } from '../lib/catalogos';

const schema = z.object({
  rfc: z.string().min(12),
  razon_social: z.string().min(1),
  regimen_fiscal: z.string(),
  cp: z.string().length(5),
  estado_isn: z.string(),
  tasa_isn: z.coerce.number(),
  prima_riesgo_trabajo: z.coerce.number(),
  registro_patronal_imss: z.string(),
});

const CLASE_CONTROL = 'border p-2 rounded w-full';

export default function EmpresaPage() {
  const { register, handleSubmit, reset } = useForm({ resolver: zodResolver(schema) });
  const [msg, setMsg] = useState('');
  const reloadKey = useAppStore((s) => s.reloadKey);

  // C4: obtener_empresa resuelve la activa en el backend.
  useEffect(() => {
    api.obtenerEmpresa().then((e: any) => { if (e) reset(e); }).catch(console.error);
  }, [reset, reloadKey]);

  const onSubmit = async (data: any) => {
    await api.guardarEmpresa(data);
    setMsg('Empresa guardada');
  };

  return (
    <form onSubmit={handleSubmit(onSubmit)} className="grid grid-cols-1 md:grid-cols-2 gap-4 max-w-4xl">
      <h1 className="col-span-full text-xl font-bold">Configuración de empresa</h1>
      <p className="col-span-full text-sm text-gray-600">
        Capture los datos fiscales de la empresa. Todos los campos muestran un ejemplo y una
        explicación debajo; los catálogos se eligen de una lista.
      </p>

      <Campo
        id="empresa-rfc"
        label="RFC de la empresa"
        hint="13 caracteres para persona moral. Para pruebas use XAXX010101000."
      >
        <input {...register('rfc')} className={CLASE_CONTROL} placeholder="XAXX010101000" />
      </Campo>

      <Campo
        id="empresa-razon-social"
        label="Razón social"
        hint="Nombre legal de la empresa, tal como aparece en su constancia de situación fiscal."
      >
        <input {...register('razon_social')} className={CLASE_CONTROL} placeholder="Mi Empresa S.A. de C.V." />
      </Campo>

      <Campo
        id="empresa-regimen-fiscal"
        label="Régimen fiscal (clave SAT)"
        hint="601 = General de Ley Personas Morales, 605 = Sueldos y Salarios, 612 = Personas Físicas con Actividades Empresariales"
      >
        <select {...register('regimen_fiscal')} className={CLASE_CONTROL}>
          {REGIMENES_FISCALES.map((r) => (
            <option key={r.value} value={r.value}>{r.label}</option>
          ))}
        </select>
      </Campo>

      <Campo
        id="empresa-cp"
        label="Código postal"
        hint="5 dígitos del domicilio fiscal de la empresa. Ejemplo: 01000."
      >
        <input {...register('cp')} inputMode="numeric" className={CLASE_CONTROL} placeholder="01000" />
      </Campo>

      <Campo
        id="empresa-estado-isn"
        label="Estado de la República"
        hint="Entidad federativa donde se calcula el ISN. Elija de la lista; no es texto libre."
      >
        <select {...register('estado_isn')} className={CLASE_CONTROL}>
          <option value="">Seleccione un estado…</option>
          {ESTADOS_REPUBLICA.map((estado) => (
            <option key={estado} value={estado}>{estado}</option>
          ))}
        </select>
      </Campo>

      <Campo
        id="empresa-tasa-isn"
        label="Tasa de ISN (%)"
        hint="Porcentaje del impuesto sobre nóminas de esa entidad. Ejemplo: 3 para 3%."
      >
        <input {...register('tasa_isn')} type="number" step="0.01" min="0" className={CLASE_CONTROL} placeholder="3" />
      </Campo>

      <Campo
        id="empresa-prima-riesgo"
        label="Prima de riesgo de trabajo (%)"
        hint="Prima anual de riesgo de trabajo. Oscila entre 0.5% (mínima) y 15% (máxima). Si es la primera vez, usar 0.5%."
      >
        <input {...register('prima_riesgo_trabajo')} type="number" step="0.01" min="0.5" max="15" className={CLASE_CONTROL} placeholder="0.5" />
      </Campo>

      <Campo
        id="empresa-registro-patronal"
        label="Registro patronal IMSS"
        hint="Opcional. Si no lo tienes, dejar vacío."
      >
        <input {...register('registro_patronal_imss')} className={CLASE_CONTROL} placeholder="A1234567890" />
      </Campo>

      <button type="submit" className="col-span-full bg-blue-600 text-white p-2 rounded">Guardar empresa</button>
      {msg && <span className="col-span-full text-green-600">{msg}</span>}
    </form>
  );
}
