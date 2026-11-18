import { useEffect, useState } from 'react';
import { api } from '../lib/tauri';
import { useAppStore } from '../lib/store';
import { BotonDescargarRecibo } from '../components/ReciboPDF';

export default function NominaPage() {
  const [trabajadores, setTrabajadores] = useState<any[]>([]);
  const [periodoId, setPeriodoId] = useState<number | null>(null);
  const [resultados, setResultados] = useState<any[]>([]);
  const [tipo, setTipo] = useState('mensual');
  const [fechaInicio, setFechaInicio] = useState('');
  const [fechaFin, setFechaFin] = useState('');
  const [fechaPago, setFechaPago] = useState('');
  const [dias, setDias] = useState(30);
  const [seleccionados, setSeleccionados] = useState<number[]>([]);
  // Bloque B: captura opcional de tiempo extraordinario.
  const [horasExtra, setHorasExtra] = useState(0);
  const [semanasExtra, setSemanasExtra] = useState(1);
  const [jornadaExtra, setJornadaExtra] = useState('diurna');
  const [salarioMinimo, setSalarioMinimo] = useState(false);
  const reloadKey = useAppStore((s) => s.reloadKey);

  // C4: al cambiar de empresa se recargan los trabajadores y se limpia la selección.
  useEffect(() => {
    setSeleccionados([]);
    setResultados([]);
    setPeriodoId(null);
    api.listarTrabajadores(1).then(setTrabajadores).catch(console.error);
  }, [reloadKey]);

  const crearPeriodo = async () => {
    const id = await api.guardarPeriodo({
      empresa_id: 1, tipo, fecha_inicio: fechaInicio, fecha_fin: fechaFin, fecha_pago: fechaPago, estado: 'borrador', dias,
    });
    setPeriodoId(id as number);
  };

  const calcular = async () => {
    const out: any[] = [];
    for (const id of seleccionados) {
      const horasExtraInput = horasExtra > 0 ? {
        horas: horasExtra,
        dias,
        semanas: semanasExtra,
        jornada: jornadaExtra,
        salario_minimo: salarioMinimo,
      } : undefined;
      const r = await api.calcularRecibo(id, 1, dias, tipo, fechaPago || '2026-02-28', horasExtraInput);
      if (periodoId) await api.guardarRecibo(periodoId, r);
      // El nombre no forma parte del payload que se guarda en la BD; se agrega
      // solo a la copia que se muestra y que se envía al generador de PDF.
      const nombre = trabajadores.find((t: any) => t.id === id)?.nombre ?? '';
      out.push({ ...r, trabajador_nombre: nombre });
    }
    setResultados(out);
  };

  const toggle = (id: number) => {
    setSeleccionados((s) => s.includes(id) ? s.filter((x) => x !== id) : [...s, id]);
  };

  return (
    <div>
      <h1 className="text-xl font-bold mb-4">Captura de periodo de nomina</h1>
      <div className="grid grid-cols-4 gap-3 mb-4">
        <select value={tipo} onChange={(e) => setTipo(e.target.value)} className="border p-2 rounded">
          <option value="semanal">Semanal</option>
          <option value="quincenal">Quincenal</option>
          <option value="mensual">Mensual</option>
        </select>
        <input type="date" value={fechaInicio} onChange={(e) => setFechaInicio(e.target.value)} className="border p-2 rounded" />
        <input type="date" value={fechaFin} onChange={(e) => setFechaFin(e.target.value)} className="border p-2 rounded" />
        <input type="date" value={fechaPago} onChange={(e) => setFechaPago(e.target.value)} className="border p-2 rounded" />
        <input type="number" value={dias} onChange={(e) => setDias(Number(e.target.value))} className="border p-2 rounded" placeholder="Dias" />
      </div>
      <div className="grid grid-cols-4 gap-3 mb-4 border-t pt-3">
        <input type="number" value={horasExtra} onChange={(e) => setHorasExtra(Number(e.target.value))} className="border p-2 rounded" placeholder="Horas extra" min="0" step="0.5" />
        <input type="number" value={semanasExtra} onChange={(e) => setSemanasExtra(Number(e.target.value))} className="border p-2 rounded" placeholder="Semanas con extras" min="1" />
        <select value={jornadaExtra} onChange={(e) => setJornadaExtra(e.target.value)} className="border p-2 rounded">
          <option value="diurna">Jornada diurna</option>
          <option value="nocturna">Jornada nocturna</option>
          <option value="mixta">Jornada mixta</option>
        </select>
        <label className="flex items-center gap-2">
          <input type="checkbox" checked={salarioMinimo} onChange={(e) => setSalarioMinimo(e.target.checked)} />
          Salario mínimo (ISR 93 I)
        </label>
      </div>
      <button onClick={crearPeriodo} className="bg-gray-600 text-white p-2 rounded mb-4">Crear periodo</button>
      {periodoId && <span className="ml-2">Periodo #{periodoId}</span>}

      <h2 className="font-bold mt-4">Seleccionar trabajadores</h2>
      <div className="space-y-1 mb-4">
        {trabajadores.map((t: any) => (
          <label key={t.id} className="flex items-center gap-2">
            <input type="checkbox" checked={seleccionados.includes(t.id)} onChange={() => toggle(t.id)} />
            {t.nombre} - {t.salario_diario}
          </label>
        ))}
      </div>
      <button onClick={calcular} className="bg-blue-600 text-white p-2 rounded">Calcular nomina</button>

      {resultados.length > 0 && (
        <table className="w-full border mt-6 text-sm">
          <thead className="bg-gray-100">
            <tr>
              <th>Trab</th><th>Percepciones</th><th>Hras dobles</th><th>Hras triples</th><th>HE exento</th><th>ISR</th><th>IMSS</th><th>INFONAVIT</th><th>Neto</th><th>Costo patronal</th><th>PDF</th>
            </tr>
          </thead>
          <tbody>
            {resultados.map((r: any, i: number) => (
              <tr key={i} className="border-t">
                <td className="p-1">{r.trabajador_id}</td>
                <td className="p-1">{r.total_percepciones.toFixed(2)}</td>
                <td className="p-1">{r.horas_extra_dobles.toFixed(2)}</td>
                <td className="p-1">{r.horas_extra_triples.toFixed(2)}</td>
                <td className="p-1">{r.horas_extra_exento.toFixed(2)}</td>
                <td className="p-1">{r.isr.toFixed(2)}</td>
                <td className="p-1">{r.imss_obrero.toFixed(2)}</td>
                <td className="p-1">{r.infonavit_desc.toFixed(2)}</td>
                <td className="p-1">{r.neto.toFixed(2)}</td>
                <td className="p-1">{r.costo_total_patron.toFixed(2)}</td>
                <td className="p-1"><BotonDescargarRecibo recibo={r} /></td>
              </tr>
            ))}
          </tbody>
        </table>
      )}
    </div>
  );
}