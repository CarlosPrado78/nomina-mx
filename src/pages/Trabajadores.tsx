import { useEffect, useState } from 'react';
import { useForm } from 'react-hook-form';
import { zodResolver } from '@hookform/resolvers/zod';
import { z } from 'zod';
import { api } from '../lib/tauri';
import { useAppStore } from '../lib/store';
import { validarCURP, validarRFC, validarNSS } from '../lib/validators';

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

export default function TrabajadoresPage() {
  const { register, handleSubmit, reset } = useForm({ resolver: zodResolver(schema), defaultValues: { empresa_id: 1, nombre: '', curp: '', rfc: '', nss: '', num_empleado: '', fecha_nacimiento: '', estado_civil: '', fecha_ingreso: '', tipo_contrato: '', departamento: '', puesto: '', salario_diario: 0, periodicidad_pago: 'mensual', jornada: '', horas_semanales: 40, infonavit_activo: false, infonavit_num_credito: '', infonavit_factor: 0, infonavit_tipo_descuento: 'CUOTA_FIJA_VSM' } });
  const [lista, setLista] = useState<any[]>([]);
  const [msg, setMsg] = useState('');
  const reloadKey = useAppStore((s) => s.reloadKey);

  const cargar = async () => {
    try { setLista(await api.listarTrabajadores(1)); } catch (e) { console.error(e); }
  };

  // C4: recarga cuando cambia la empresa activa.
  useEffect(() => { cargar(); }, [reloadKey]);

  const onSubmit = async (data: any) => {
    await api.guardarTrabajador(data);
    setMsg('Trabajador guardado');
    reset({ empresa_id: 1, infonavit_activo: false });
    cargar();
  };

  return (
    <div>
      <h1 className="text-xl font-bold mb-4">Alta de trabajador</h1>
      <form onSubmit={handleSubmit(onSubmit)} className="grid grid-cols-3 gap-3 mb-6">
        <input type="hidden" {...register('empresa_id')} />
        <input placeholder="Nombre" {...register('nombre')} className="border p-2 rounded" />
        <input placeholder="CURP" {...register('curp')} className="border p-2 rounded" />
        <input placeholder="RFC" {...register('rfc')} className="border p-2 rounded" />
        <input placeholder="NSS" {...register('nss')} className="border p-2 rounded" />
        <input placeholder="Num empleado" {...register('num_empleado')} className="border p-2 rounded" />
        <input placeholder="Fecha nacimiento" type="date" {...register('fecha_nacimiento')} className="border p-2 rounded" />
        <input placeholder="Estado civil" {...register('estado_civil')} className="border p-2 rounded" />
        <input placeholder="Fecha ingreso" type="date" {...register('fecha_ingreso')} className="border p-2 rounded" />
        <input placeholder="Tipo contrato" {...register('tipo_contrato')} className="border p-2 rounded" />
        <input placeholder="Departamento" {...register('departamento')} className="border p-2 rounded" />
        <input placeholder="Puesto" {...register('puesto')} className="border p-2 rounded" />
        <input placeholder="Salario diario" type="number" step="0.01" {...register('salario_diario')} className="border p-2 rounded" />
        <select {...register('periodicidad_pago')} className="border p-2 rounded">
          <option value="semanal">Semanal</option>
          <option value="quincenal">Quincenal</option>
          <option value="mensual">Mensual</option>
        </select>
        <input placeholder="Jornada" {...register('jornada')} className="border p-2 rounded" />
        <input placeholder="Horas semanales" type="number" {...register('horas_semanales')} className="border p-2 rounded" />
        <label className="flex items-center gap-2"><input type="checkbox" {...register('infonavit_activo')} /> INFONAVIT activo</label>
        <input placeholder="Num credito INFONAVIT" {...register('infonavit_num_credito')} className="border p-2 rounded" />
        <input placeholder="Factor INFONAVIT" type="number" step="0.01" {...register('infonavit_factor')} className="border p-2 rounded" />
        <select {...register('infonavit_tipo_descuento')} className="border p-2 rounded">
          <option value="CUOTA_FIJA_VSM">Cuota fija VSM</option>
          <option value="PORCENTAJE">Porcentaje</option>
          <option value="MONTO">Monto</option>
        </select>
        <button type="submit" className="col-span-3 bg-blue-600 text-white p-2 rounded">Guardar trabajador</button>
        {msg && <span className="col-span-3 text-green-600">{msg}</span>}
      </form>

      <h2 className="text-lg font-bold mb-2">Trabajadores</h2>
      <table className="w-full border">
        <thead><tr className="bg-gray-100"><th>Nombre</th><th>RFC</th><th>Salario</th></tr></thead>
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