import { useEffect, useState } from 'react';
import { api } from '../lib/tauri';
import { useAppStore } from '../lib/store';

export default function ReportesPage() {
  const [data, setData] = useState<any[]>([]);
  const reloadKey = useAppStore((s) => s.reloadKey);
  // C4: el backend filtra por empresa activa; se recarga al cambiar de empresa.
  useEffect(() => { api.reporteCuotasImss(1).then(setData).catch(console.error); }, [reloadKey]);
  return (
    <div>
      <h1 className="text-xl font-bold mb-4">Reporte cuotas IMSS (SUA)</h1>
      <table className="w-full border">
        <thead className="bg-gray-100"><tr><th>Periodo</th><th>Obrero</th><th>Patronal</th></tr></thead>
        <tbody>
          {data.map((r: any, i: number) => (
            <tr key={i} className="border-t">
              <td className="p-2">{r.periodo_id}</td>
              <td className="p-2">{r.total_obrero?.toFixed(2)}</td>
              <td className="p-2">{r.total_patronal?.toFixed(2)}</td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}