import { useState } from 'react';
import { api } from '../lib/tauri';

export default function FiniquitosPage() {
  const [salario, setSalario] = useState(278.80);
  const [diasPend, setDiasPend] = useState(0);
  const [diasVac, setDiasVac] = useState(0);
  const [diasTrab, setDiasTrab] = useState(180);
  const [anios, setAnios] = useState(2);
  const [tipo, setTipo] = useState('finiquito');
  const [res, setRes] = useState<any>(null);

  const calcular = async () => {
    if (tipo === 'finiquito') {
      const r = await api.calcularFiniquito({
        salarioDiario: salario, diasPendientes: diasPend, diasVacacionesPendientes: diasVac,
        primaVacPct: 0.25, diasTrabajadosAnio: diasTrab, diasAguinaldoMinimos: 15,
      });
      setRes(r);
    } else {
      const finBase = (await api.calcularFiniquito({
        salarioDiario: salario, diasPendientes: diasPend, diasVacacionesPendientes: diasVac,
        primaVacPct: 0.25, diasTrabajadosAnio: diasTrab, diasAguinaldoMinimos: 15,
      })) as any;
      const r = await api.calcularLiquidacion({
        salarioDiario: salario, aniosAntiguedad: anios, finiquitoBase: finBase.total, salarioMinimo: 278.80,
      });
      setRes({ ...r, finiquito_base: finBase.total });
    }
  };

  return (
    <div>
      <h1 className="text-xl font-bold mb-4">Finiquitos y liquidaciones</h1>
      <div className="grid grid-cols-3 gap-3 mb-4">
        <select value={tipo} onChange={(e) => setTipo(e.target.value)} className="border p-2 rounded">
          <option value="finiquito">Finiquito (renuncia)</option>
          <option value="liquidacion">Liquidacion (despido injustificado)</option>
        </select>
        <input type="number" value={salario} onChange={(e) => setSalario(Number(e.target.value))} placeholder="Salario diario" className="border p-2 rounded" />
        <input type="number" value={diasPend} onChange={(e) => setDiasPend(Number(e.target.value))} placeholder="Dias pendientes" className="border p-2 rounded" />
        <input type="number" value={diasVac} onChange={(e) => setDiasVac(Number(e.target.value))} placeholder="Dias vacaciones pendientes" className="border p-2 rounded" />
        <input type="number" value={diasTrab} onChange={(e) => setDiasTrab(Number(e.target.value))} placeholder="Dias trabajados anio" className="border p-2 rounded" />
        <input type="number" value={anios} onChange={(e) => setAnios(Number(e.target.value))} placeholder="Anios antiguedad" className="border p-2 rounded" />
      </div>
      <button onClick={calcular} className="bg-blue-600 text-white p-2 rounded">Calcular</button>
      {res && (
        <pre className="mt-4 bg-white p-4 rounded shadow text-sm">{JSON.stringify(res, null, 2)}</pre>
      )}
    </div>
  );
}