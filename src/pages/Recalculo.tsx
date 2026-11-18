import { useEffect, useState } from 'react';
import { api, type ResultadoRecalculo } from '../lib/tauri';
import { useAppStore } from '../lib/store';

interface Periodo {
  id: number;
  fecha_inicio: string;
  fecha_fin: string;
  fecha_pago: string;
  estado: string;
  dias: number;
}

interface Trabajador {
  id: number;
  nombre: string;
}

const dinero = new Intl.NumberFormat('es-MX', {
  style: 'currency',
  currency: 'MXN',
  minimumFractionDigits: 2,
});

/**
 * C5 — Recálculo retroactivo. El motivo es obligatorio en la ruta de periodo;
 * los parámetros se resuelven en Rust con la fecha de pago original.
 */
export default function RecalculoPage() {
  const [periodos, setPeriodos] = useState<Periodo[]>([]);
  const [trabajadores, setTrabajadores] = useState<Trabajador[]>([]);
  const [periodoId, setPeriodoId] = useState<number | null>(null);
  const [trabajadorId, setTrabajadorId] = useState<number | null>(null);
  const [motivo, setMotivo] = useState('');
  const [resultado, setResultado] = useState<ResultadoRecalculo | null>(null);
  const [error, setError] = useState('');
  const [procesando, setProcesando] = useState(false);
  const reloadKey = useAppStore((s) => s.reloadKey);

  useEffect(() => {
    let activo = true;
    Promise.all([
      api.listarPeriodos(1),
      api.listarTrabajadores(1),
    ])
      .then(([ps, ts]) => {
        if (!activo) return;
        setPeriodos(ps as Periodo[]);
        setTrabajadores(ts as Trabajador[]);
        setPeriodoId(null);
        setTrabajadorId(null);
        setResultado(null);
      })
      .catch((e) => activo && setError(String(e)));
    return () => {
      activo = false;
    };
  }, [reloadKey]);

  const periodo = periodos.find((p) => p.id === periodoId) ?? null;
  const motivoValido = motivo.trim().length > 0;

  const ejecutar = async (individual: boolean) => {
    if (!periodoId || (!individual && !motivoValido)) return;
    setProcesando(true);
    setError('');
    setResultado(null);
    try {
      const res = individual
        ? await api.recalcularTrabajadorPeriodo(trabajadorId as number, periodoId)
        : await api.recalcularPeriodo(periodoId, motivo.trim());
      setResultado(res);
    } catch (e) {
      setError(String(e));
    } finally {
      setProcesando(false);
    }
  };

  return (
    <div className="max-w-5xl">
      <h1 className="text-xl font-bold mb-1">Recálculo retroactivo de SBC y recibos</h1>
      <p className="text-sm text-gray-600 mb-4">
        Regenera los recibos con los parámetros vigentes a la <strong>fecha de pago original</strong>.
        Un periodo de enero 2026 usa UMA <strong>113.14</strong> y subsidio <strong>536.21</strong>,
        aunque hoy sea noviembre. El motivo queda registrado en <code>audit_log</code> como
        <code> RECALCULO</code>.
      </p>

      <div className="grid md:grid-cols-2 gap-4 mb-4">
        <label className="block">
          <span className="text-sm font-semibold">Periodo</span>
          <select
            className="border rounded p-2 w-full"
            value={periodoId ?? ''}
            onChange={(e) => {
              setPeriodoId(e.target.value ? Number(e.target.value) : null);
              setResultado(null);
            }}
          >
            <option value="">Seleccione un periodo…</option>
            {periodos.map((p) => (
              <option key={p.id} value={p.id}>
                #{p.id} · pago {p.fecha_pago} · {p.estado}
              </option>
            ))}
          </select>
        </label>
        <label className="block">
          <span className="text-sm font-semibold">Motivo (obligatorio)</span>
          <input
            className="border rounded p-2 w-full"
            value={motivo}
            onChange={(e) => setMotivo(e.target.value)}
            placeholder="Ej. Corrección de salario posterior al cierre"
          />
        </label>
      </div>

      {periodo && (
        <p className="text-sm mb-4">
          <span className="font-semibold">Detalle:</span> {periodo.fecha_inicio} a {periodo.fecha_fin}
          {' · '}{periodo.dias} días · estado <code>{periodo.estado}</code>.
          Los periodos abiertos también se pueden recalcular; el motivo sigue siendo obligatorio.
        </p>
      )}

      <div className="flex flex-wrap gap-2 mb-4">
        <button
          type="button"
          disabled={!periodoId || !motivoValido || procesando}
          onClick={() => ejecutar(false)}
          className="rounded px-3 py-2 bg-blue-600 text-white disabled:opacity-50"
        >
          {procesando ? 'Recalculando…' : 'Recalcular periodo completo'}
        </button>
        <select
          aria-label="Trabajador individual"
          className="border rounded p-2"
          value={trabajadorId ?? ''}
          onChange={(e) => setTrabajadorId(e.target.value ? Number(e.target.value) : null)}
        >
          <option value="">Trabajador (opcional)…</option>
          {trabajadores.map((t) => (
            <option key={t.id} value={t.id}>{t.nombre}</option>
          ))}
        </select>
        <button
          type="button"
          disabled={!periodoId || !trabajadorId || procesando}
          onClick={() => ejecutar(true)}
          className="rounded px-3 py-2 bg-gray-700 text-white disabled:opacity-50"
        >
          Recalcular solo este trabajador
        </button>
      </div>

      {error && (
        <div className="border border-red-300 bg-red-50 text-red-800 rounded p-3 mb-4">
          {error}
        </div>
      )}

      <div className="border-l-4 border-amber-400 bg-amber-50 p-3 text-sm mb-4">
        <p className="font-semibold">Advertencias</p>
        <ul className="list-disc ml-5 mt-1 space-y-1">
          <li>Los recibos con CFDI timbrado (<code>uuid_cfdi</code>) se reportan pero NO se modifican.</li>
          <li>El CFDI de ajuste con <code>TipoDeComprobante="E"</code> está pendiente para v1.1; esta entrega solo calcula y reporta la diferencia.</li>
          <li>La ruta individual usa un motivo de sistema no vacío porque su firma no recibe motivo.</li>
        </ul>
      </div>

      {resultado && (
        <div className="space-y-4">
          <div className="border rounded p-3 bg-green-50">
            <span className="font-semibold">Recibos actualizados:</span> {resultado.recibos_recalculados}
            {' · '}
            <span className="font-semibold">Diferencias detectadas:</span> {resultado.diferencias.length}
          </div>

          {resultado.diferencias.length > 0 && (
            <table className="w-full text-sm border-collapse">
              <thead>
                <tr className="bg-gray-100 text-left">
                  <th className="border p-2">Trabajador</th>
                  <th className="border p-2">Percepciones antes</th>
                  <th className="border p-2">Percepciones después</th>
                  <th className="border p-2">Deducciones antes</th>
                  <th className="border p-2">Deducciones después</th>
                  <th className="border p-2">Neto antes</th>
                  <th className="border p-2">Neto después</th>
                  <th className="border p-2">Δ neto</th>
                </tr>
              </thead>
              <tbody>
                {resultado.diferencias.map((d) => (
                  <tr key={d.trabajador_id}>
                    <td className="border p-2">{d.nombre}</td>
                    <td className="border p-2">{dinero.format(d.percepciones_antes)}</td>
                    <td className="border p-2">{dinero.format(d.percepciones_despues)}</td>
                    <td className="border p-2">{dinero.format(d.deducciones_antes)}</td>
                    <td className="border p-2">{dinero.format(d.deducciones_despues)}</td>
                    <td className="border p-2">{dinero.format(d.neto_antes)}</td>
                    <td className="border p-2">{dinero.format(d.neto_despues)}</td>
                    <td className={`border p-2 ${d.delta_neto >= 0 ? 'text-green-700' : 'text-red-700'}`}>
                      {dinero.format(d.delta_neto)}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          )}
        </div>
      )}
    </div>
  );
}
