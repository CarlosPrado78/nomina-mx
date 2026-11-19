import { useState } from 'react';
import { api } from '../lib/tauri';
import { Campo } from '../components/Campo';
import { TIPOS_SEPARACION } from '../lib/catalogos';

const CLASE_CONTROL = 'border p-2 rounded w-full';

export default function FiniquitosPage() {
  const [salario, setSalario] = useState(278.80);
  const [diasPend, setDiasPend] = useState(0);
  const [diasVac, setDiasVac] = useState(0);
  const [diasTrab, setDiasTrab] = useState(180);
  const [anios, setAnios] = useState(2);
  const [tipo, setTipo] = useState('finiquito');
  const [fechaBaja, setFechaBaja] = useState('');
  const [tipoSeparacion, setTipoSeparacion] = useState('renuncia');
  const [motivo, setMotivo] = useState('');
  const [res, setRes] = useState<any>(null);

  const calcular = async () => {
    if (tipo === 'finiquito') {
      const r = await api.calcularFiniquito({
        salarioDiario: salario, diasPendientes: diasPend, diasVacacionesPendientes: diasVac,
        primaVacPct: 0.25, diasTrabajadosAnio: diasTrab, diasAguinaldoMinimos: 15,
      });
      setRes({ ...r, fecha_baja: fechaBaja, tipo_separacion: tipoSeparacion, motivo });
    } else {
      const finBase = (await api.calcularFiniquito({
        salarioDiario: salario, diasPendientes: diasPend, diasVacacionesPendientes: diasVac,
        primaVacPct: 0.25, diasTrabajadosAnio: diasTrab, diasAguinaldoMinimos: 15,
      })) as any;
      const r = await api.calcularLiquidacion({
        salarioDiario: salario, aniosAntiguedad: anios, finiquitoBase: finBase.total, salarioMinimo: 278.80,
      });
      setRes({ ...r, finiquito_base: finBase.total, fecha_baja: fechaBaja, tipo_separacion: tipoSeparacion, motivo });
    }
  };

  return (
    <div>
      <h1 className="text-xl font-bold mb-1">Finiquitos y liquidaciones</h1>
      <p className="text-sm text-gray-600 mb-4">
        Capture los importes en pesos y los días naturales conforme al contrato. La fecha de baja y el
        motivo son informativos: se muestran en el resultado para su expediente.
      </p>
      <div className="grid grid-cols-1 md:grid-cols-3 gap-3 mb-4">
        <Campo
          id="finiquito-tipo-calculo"
          label="Tipo de cálculo"
          hint="Finiquito por renuncia o liquidación por despido injustificado."
        >
          <select value={tipo} onChange={(e) => setTipo(e.target.value)} className={CLASE_CONTROL}>
            <option value="finiquito">Finiquito (renuncia)</option>
            <option value="liquidacion">Liquidación (despido injustificado)</option>
          </select>
        </Campo>

        <Campo
          id="finiquito-fecha-baja"
          label="Fecha de baja"
          hint="Último día laborado del trabajador. Formato AAAA-MM-DD."
        >
          <input type="date" value={fechaBaja} onChange={(e) => setFechaBaja(e.target.value)} className={CLASE_CONTROL} />
        </Campo>

        <Campo
          id="finiquito-tipo-separacion"
          label="Tipo de separación"
          hint="Motivo legal de la baja; determina si se calcula liquidación o solo finiquito."
        >
          <select value={tipoSeparacion} onChange={(e) => setTipoSeparacion(e.target.value)} className={CLASE_CONTROL}>
            {TIPOS_SEPARACION.map((t) => <option key={t.value} value={t.value}>{t.label}</option>)}
          </select>
        </Campo>

        <Campo
          id="finiquito-salario-diario"
          label="Salario diario (pesos)"
          hint="Salario diario del trabajador, antes de impuestos. Ejemplo: 500.00."
        >
          <input type="number" value={salario} onChange={(e) => setSalario(Number(e.target.value))} className={CLASE_CONTROL} placeholder="500.00" min="0" step="0.01" />
        </Campo>

        <Campo
          id="finiquito-dias-pendientes"
          label="Días de salario pendientes (días)"
          hint="Días laborados que aún no se han pagado. Use 0 si no hay pendientes."
        >
          <input type="number" value={diasPend} onChange={(e) => setDiasPend(Number(e.target.value))} className={CLASE_CONTROL} placeholder="0" min="0" />
        </Campo>

        <Campo
          id="finiquito-dias-vacaciones"
          label="Días de vacaciones pendientes (días)"
          hint="Días de vacaciones no disfrutadas que se liquidan. Use 0 si no hay."
        >
          <input type="number" value={diasVac} onChange={(e) => setDiasVac(Number(e.target.value))} className={CLASE_CONTROL} placeholder="0" min="0" />
        </Campo>

        <Campo
          id="finiquito-dias-trabajados-anio"
          label="Días trabajados en el año (días)"
          hint="Días laborados en el año de la baja; se usa para aguinaldo y prima vacacional."
        >
          <input type="number" value={diasTrab} onChange={(e) => setDiasTrab(Number(e.target.value))} className={CLASE_CONTROL} placeholder="180" min="0" />
        </Campo>

        <Campo
          id="finiquito-anios-antiguedad"
          label="Años de antigüedad (años)"
          hint="Años laborados en la empresa; solo se usa en liquidaciones."
        >
          <input type="number" value={anios} onChange={(e) => setAnios(Number(e.target.value))} className={CLASE_CONTROL} placeholder="2" min="0" />
        </Campo>

        <Campo
          id="finiquito-motivo"
          label="Motivo (opcional)"
          hint="Descripción libre de la baja. Ejemplo: renuncia voluntaria presentada el 15 de enero."
        >
          <input value={motivo} onChange={(e) => setMotivo(e.target.value)} className={CLASE_CONTROL} placeholder="Renuncia voluntaria" />
        </Campo>
      </div>
      <button onClick={calcular} className="bg-blue-600 text-white p-2 rounded">Calcular</button>
      {res && (
        <pre className="mt-4 bg-white p-4 rounded shadow text-sm">{JSON.stringify(res, null, 2)}</pre>
      )}
    </div>
  );
}
