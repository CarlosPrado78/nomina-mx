import { useEffect, useState } from 'react';
import { api } from '../lib/tauri';

export default function ConfiguracionPage() {
  const [estados, setEstados] = useState<any[]>([]);
  useEffect(() => { api.listarEstadosIsn().then(setEstados).catch(console.error); }, []);
  return (
    <div>
      <h1 className="text-xl font-bold mb-4">Configuracion fiscal</h1>
      <p className="mb-2">Tasas ISN por estado (editable via parametros):</p>
      <table className="w-full border text-sm">
        <thead className="bg-gray-100"><tr><th>Estado</th><th>Tasa %</th></tr></thead>
        <tbody>
          {estados.map((e: any, i: number) => (
            <tr key={i} className="border-t"><td className="p-2">{e.estado}</td><td className="p-2">{e.tasa}</td></tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}