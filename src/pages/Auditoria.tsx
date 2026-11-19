import { useEffect, useState } from 'react';
import { api } from '../lib/tauri';
import { Campo } from '../components/Campo';

interface RegistroAuditoria {
  id: number;
  tabla: string;
  operacion: string;
  registro_id: number;
  antes_json: string | null;
  despues_json: string | null;
  usuario: string;
  timestamp: string;
}

interface Filtros {
  tabla?: string;
  operacion?: string;
  usuario?: string;
  fecha_desde?: string;
  fecha_hasta?: string;
}

const TABLAS = [
  'trabajadores',
  'recibos',
  'periodos_nomina',
  'incapacidades',
  'vacaciones',
];
const OPERACIONES = ['INSERT', 'UPDATE', 'DELETE', 'RECALCULO'];
const PAGE_SIZE = 10;

function resumen(json: string | null): string {
  if (!json) return '—';
  try {
    const valor = JSON.parse(json);
    const campos = Object.keys(valor).length;
    return `${campos} campos · ${json.slice(0, 60)}${json.length > 60 ? '…' : ''}`;
  } catch {
    return json.slice(0, 80);
  }
}

export default function AuditoriaPage() {
  const [filtros, setFiltros] = useState<Filtros>({});
  const [pagina, setPagina] = useState(1);
  const [registros, setRegistros] = useState<RegistroAuditoria[]>([]);
  const [error, setError] = useState('');
  const [cargando, setCargando] = useState(false);

  useEffect(() => {
    let activo = true;
    setCargando(true);
    api
      .listarAuditoria({
        ...filtros,
        limit: PAGE_SIZE,
        offset: (pagina - 1) * PAGE_SIZE,
      })
      .then((respuesta) => {
        if (activo) {
          setRegistros(respuesta as RegistroAuditoria[]);
          setError('');
        }
      })
      .catch((e) => {
        if (activo) setError(String(e));
      })
      .finally(() => {
        if (activo) setCargando(false);
      });
    return () => {
      activo = false;
    };
  }, [filtros, pagina]);

  const cambiar = (clave: keyof Filtros, valor: string) => {
    setPagina(1);
    setFiltros((anteriores) => ({ ...anteriores, [clave]: valor || undefined }));
  };

  return (
    <div>
      <h1 className="text-xl font-bold mb-1">Log de auditoría</h1>
      <p className="text-sm text-gray-600 mb-4">
        Registros inmutables (UTC). Incluye altas, cambios, bajas y recálculos.
      </p>

      <div className="grid grid-cols-2 md:grid-cols-5 gap-2 mb-4">
        <Campo
          id="auditoria-tabla"
          label="Tabla"
          hint="Tabla de la base de datos donde ocurrió el cambio."
        >
          <select
            className="border rounded p-2 w-full"
            value={filtros.tabla ?? ''}
            onChange={(e) => cambiar('tabla', e.target.value)}
          >
            <option value="">Todas las tablas</option>
            {TABLAS.map((t) => (
              <option key={t} value={t}>{t}</option>
            ))}
          </select>
        </Campo>
        <Campo
          id="auditoria-operacion"
          label="Operación"
          hint="Alta (INSERT), cambio (UPDATE), baja (DELETE) o recálculo (RECALCULO)."
        >
          <select
            className="border rounded p-2 w-full"
            value={filtros.operacion ?? ''}
            onChange={(e) => cambiar('operacion', e.target.value)}
          >
            <option value="">Todas las operaciones</option>
            {OPERACIONES.map((o) => (
              <option key={o} value={o}>{o}</option>
            ))}
          </select>
        </Campo>
        <Campo
          id="auditoria-usuario"
          label="Usuario"
          hint="Usuario de auditoría que hizo el cambio. Ejemplo: system."
        >
          <input
            className="border rounded p-2 w-full"
            placeholder="system"
            value={filtros.usuario ?? ''}
            onChange={(e) => cambiar('usuario', e.target.value)}
          />
        </Campo>
        <Campo
          id="auditoria-desde"
          label="Fecha desde"
          hint="Primer día del rango, en UTC. Formato AAAA-MM-DD."
        >
          <input
            type="date"
            className="border rounded p-2 w-full"
            value={filtros.fecha_desde ?? ''}
            onChange={(e) => cambiar('fecha_desde', e.target.value)}
          />
        </Campo>
        <Campo
          id="auditoria-hasta"
          label="Fecha hasta"
          hint="Último día del rango, en UTC. Formato AAAA-MM-DD."
        >
          <input
            type="date"
            className="border rounded p-2 w-full"
            value={filtros.fecha_hasta ?? ''}
            onChange={(e) => cambiar('fecha_hasta', e.target.value)}
          />
        </Campo>
      </div>

      {error && <p className="text-red-600 mb-2">{error}</p>}
      {cargando && <p className="text-gray-500 mb-2">Cargando…</p>}

      <div className="overflow-x-auto">
        <table className="w-full border text-sm">
          <thead className="bg-gray-100">
            <tr>
              <th className="p-2 text-left">Fecha (UTC)</th>
              <th className="p-2 text-left">Tabla</th>
              <th className="p-2 text-left">Operación</th>
              <th className="p-2 text-left">Registro</th>
              <th className="p-2 text-left">Usuario</th>
              <th className="p-2 text-left">Antes</th>
              <th className="p-2 text-left">Después</th>
            </tr>
          </thead>
          <tbody>
            {registros.map((r) => (
              <tr key={r.id} className="border-t align-top">
                <td className="p-2 whitespace-nowrap">{r.timestamp}</td>
                <td className="p-2">{r.tabla}</td>
                <td className="p-2">{r.operacion}</td>
                <td className="p-2">{r.registro_id}</td>
                <td className="p-2">{r.usuario}</td>
                <td className="p-2">{resumen(r.antes_json)}</td>
                <td className="p-2">{resumen(r.despues_json)}</td>
              </tr>
            ))}
            {!cargando && registros.length === 0 && (
              <tr>
                <td colSpan={7} className="p-4 text-center text-gray-500">
                  Sin registros para los filtros seleccionados.
                </td>
              </tr>
            )}
          </tbody>
        </table>
      </div>

      <div className="flex items-center gap-3 mt-4">
        <button
          className="border rounded px-3 py-1 disabled:opacity-40"
          disabled={pagina === 1}
          onClick={() => setPagina((p) => Math.max(1, p - 1))}
        >
          Anterior
        </button>
        <span className="text-sm">Página {pagina}</span>
        <button
          className="border rounded px-3 py-1 disabled:opacity-40"
          disabled={registros.length < PAGE_SIZE}
          onClick={() => setPagina((p) => p + 1)}
        >
          Siguiente
        </button>
      </div>
    </div>
  );
}
