import { useEffect, useState } from 'react';
import { api } from '../lib/tauri';
import { useAppStore } from '../lib/store';
import { BotonDescargarRecibo } from '../components/ReciboPDF';
import { Campo } from '../components/Campo';
import { JORNADAS_LABORALES, TIPOS_PERIODO } from '../lib/catalogos';

const CLASE_CONTROL = 'border p-2 rounded w-full';

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

  const cambiarTipo = (valor: string) => {
    setTipo(valor);
    const configurado = TIPOS_PERIODO.find((p) => p.value === valor);
    if (configurado) setDias(configurado.dias);
  };

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
      <h1 className="text-xl font-bold mb-1">Captura de periodo de nómina</h1>
      <p className="text-sm text-gray-600 mb-4">
        Capture primero el periodo y luego elija a los trabajadores. Todas las fechas van en
        formato AAAA-MM-DD y los importes en pesos mexicanos.
      </p>

      <div className="grid grid-cols-1 md:grid-cols-4 gap-3 mb-4">
        <Campo
          id="periodo-tipo"
          label="Tipo de periodo"
          hint="Semanal = 7 días, quincenal = 15, decenal = 10 y mensual = 30."
        >
          <select value={tipo} onChange={(e) => cambiarTipo(e.target.value)} className={CLASE_CONTROL}>
            {TIPOS_PERIODO.map((p) => <option key={p.value} value={p.value}>{p.label}</option>)}
          </select>
        </Campo>

        <Campo
          id="periodo-fecha-inicio"
          label="Fecha de inicio del periodo"
          hint="Primer día del periodo que se va a pagar. Ej. si la quincena es del 1 al 15 de enero, esta fecha es 2026-01-01."
        >
          <input type="date" value={fechaInicio} onChange={(e) => setFechaInicio(e.target.value)} className={CLASE_CONTROL} />
        </Campo>

        <Campo
          id="periodo-fecha-fin"
          label="Fecha de fin del periodo"
          hint="Último día del periodo que se va a pagar. Ej. si la quincena es del 1 al 15 de enero, esta fecha es 2026-01-15."
        >
          <input type="date" value={fechaFin} onChange={(e) => setFechaFin(e.target.value)} className={CLASE_CONTROL} />
        </Campo>

        <Campo
          id="periodo-fecha-pago"
          label="Fecha de pago"
          hint="Día en que se le paga al trabajador. Puede ser igual o posterior a la fecha de fin del periodo."
        >
          <input type="date" value={fechaPago} onChange={(e) => setFechaPago(e.target.value)} className={CLASE_CONTROL} />
        </Campo>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-4 gap-3 mb-4 border-t pt-3">
        <Campo
          id="periodo-dias"
          label="Días pagados (calculado)"
          hint="Calculado automáticamente a partir del tipo de periodo (quincenal = 15 días). No se edita."
        >
          <input readOnly value={dias} className={`${CLASE_CONTROL} bg-gray-100`} placeholder="15" />
        </Campo>

        <Campo
          id="periodo-horas-extra"
          label="Horas extraordinarias (horas)"
          hint="Total de horas extra del periodo. Use 0 si no hay. Se pagan dobles o triples según el día."
        >
          <input type="number" value={horasExtra} onChange={(e) => setHorasExtra(Number(e.target.value))} className={CLASE_CONTROL} placeholder="0" min="0" step="0.5" />
        </Campo>

        <Campo
          id="periodo-semanas-extra"
          label="Semanas con horas extra"
          hint="Número de semanas del periodo en que se repitieron esas horas. Jornada completa = 1."
        >
          <input type="number" value={semanasExtra} onChange={(e) => setSemanasExtra(Number(e.target.value))} className={CLASE_CONTROL} placeholder="1" min="1" />
        </Campo>

        <Campo
          id="periodo-jornada-extra"
          label="Jornada de las horas extra"
          hint="Turno en que se trabajaron: diurna, nocturna o mixta; define el factor de pago."
        >
          <select value={jornadaExtra} onChange={(e) => setJornadaExtra(e.target.value)} className={CLASE_CONTROL}>
            {JORNADAS_LABORALES.filter((j) => ['diurna', 'nocturna', 'mixta'].includes(j.value)).map((j) => (
              <option key={j.value} value={j.value}>{j.label}</option>
            ))}
          </select>
        </Campo>

        <Campo
          id="periodo-salario-minimo"
          label="Horas extra al salario mínimo (ISR 93 I)"
          hint="Marque la casilla solo si las horas extra se pagan con el salario mínimo y aplica la exención del artículo 93 fracción I de la LISR."
        >
          <input type="checkbox" checked={salarioMinimo} onChange={(e) => setSalarioMinimo(e.target.checked)} className="h-4 w-4" />
        </Campo>
      </div>

      <button onClick={crearPeriodo} className="bg-gray-600 text-white p-2 rounded mb-4">Crear periodo</button>
      {periodoId && <span className="ml-2">Periodo #{periodoId}</span>}

      <div className="mt-4">
        <p id="trabajadores-a-incluir-label" className="font-bold">Trabajadores a incluir</p>
        <p id="trabajadores-a-incluir-hint" className="text-xs text-gray-500 mb-2">
          Marque cada trabajador que debe aparecer en este periodo. Puede seleccionar varios.
        </p>
        <div className="space-y-1 mb-4" role="group" aria-labelledby="trabajadores-a-incluir-label" aria-describedby="trabajadores-a-incluir-hint">
          {trabajadores.map((t: any) => (
            <div key={t.id} className="flex flex-col">
              <label htmlFor={`trabajador-${t.id}`} className="flex items-center gap-2">
                <input
                  id={`trabajador-${t.id}`}
                  aria-describedby={`trabajador-${t.id}-hint`}
                  type="checkbox"
                  checked={seleccionados.includes(t.id)}
                  onChange={() => toggle(t.id)}
                />
                {t.nombre} — salario diario {t.salario_diario} pesos
              </label>
              <p id={`trabajador-${t.id}-hint`} className="text-xs text-gray-500 ml-6">
                Se incluirá en el cálculo de este periodo si está marcado.
              </p>
            </div>
          ))}
        </div>
      </div>

      <button onClick={calcular} className="bg-blue-600 text-white p-2 rounded">Calcular nómina</button>

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
