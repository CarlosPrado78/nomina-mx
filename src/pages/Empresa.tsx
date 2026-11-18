import { useEffect, useState } from 'react';
import { useForm } from 'react-hook-form';
import { zodResolver } from '@hookform/resolvers/zod';
import { z } from 'zod';
import { api } from '../lib/tauri';
import { useAppStore } from '../lib/store';

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
    <form onSubmit={handleSubmit(onSubmit)} className="grid grid-cols-2 gap-4 max-w-3xl">
      <h1 className="col-span-2 text-xl font-bold">Configuracion de empresa</h1>
      <input placeholder="RFC" {...register('rfc')} className="border p-2 rounded" />
      <input placeholder="Razon social" {...register('razon_social')} className="border p-2 rounded" />
      <input placeholder="Regimen fiscal" {...register('regimen_fiscal')} className="border p-2 rounded" />
      <input placeholder="CP" {...register('cp')} className="border p-2 rounded" />
      <input placeholder="Estado ISN" {...register('estado_isn')} className="border p-2 rounded" />
      <input placeholder="Tasa ISN %" {...register('tasa_isn')} className="border p-2 rounded" />
      <input placeholder="Prima riesgo trabajo %" {...register('prima_riesgo_trabajo')} className="border p-2 rounded" />
      <input placeholder="Registro patronal IMSS" {...register('registro_patronal_imss')} className="border p-2 rounded" />
      <button type="submit" className="col-span-2 bg-blue-600 text-white p-2 rounded">Guardar</button>
      {msg && <span className="col-span-2 text-green-600">{msg}</span>}
    </form>
  );
}