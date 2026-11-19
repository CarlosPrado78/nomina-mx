import { useEffect, useState } from 'react';
import { api } from '../lib/tauri';
import { useAppStore } from '../lib/store';
import { Campo } from '../components/Campo';

interface Empresa {
  id: number | null;
  rfc: string;
  razon_social: string;
  activa: boolean;
}

/**
 * Selector de empresa del header (ÍTEM C4).
 * La selección se persiste en el backend (config.empresa_activa) y, al cambiar,
 * se incrementa `reloadKey` para que todas las pantallas recarguen sus datos.
 */
export default function EmpresaSelector() {
  const [empresas, setEmpresas] = useState<Empresa[]>([]);
  const [activaId, setActivaId] = useState<number | null>(null);
  const [error, setError] = useState('');
  const [mostrarAlta, setMostrarAlta] = useState(false);
  const [nueva, setNueva] = useState({ rfc: '', razon_social: '' });
  const recargar = useAppStore((s) => s.recargar);
  const setEmpresaActiva = useAppStore((s) => s.setEmpresaActiva);

  const cargar = async () => {
    try {
      const lista = (await api.listarEmpresas()) as Empresa[];
      setEmpresas(lista);
      const activa = lista.find((e) => e.activa) ?? lista[0] ?? null;
      setActivaId(activa?.id ?? null);
      setEmpresaActiva(activa?.id ?? null);
      setError('');
    } catch (e) {
      setError(String(e));
    }
  };

  useEffect(() => {
    cargar();
  }, []);

  const cambiar = async (id: number) => {
    try {
      await api.cambiarEmpresaActiva(id);
      setActivaId(id);
      setEmpresaActiva(id);
      setEmpresas((prev) => prev.map((e) => ({ ...e, activa: e.id === id })));
      setError('');
      recargar();
    } catch (e) {
      setError(String(e));
    }
  };

  const crear = async () => {
    try {
      const creada = (await api.crearEmpresa({
        rfc: nueva.rfc,
        razon_social: nueva.razon_social,
        regimen_fiscal: '601',
        cp: '00000',
        estado_isn: 'CIUDAD DE MEXICO',
        tasa_isn: 3,
        prima_riesgo_trabajo: 0.5,
        registro_patronal_imss: '',
      })) as Empresa;
      await cargar();
      if (creada.id) await cambiar(creada.id);
      setMostrarAlta(false);
      setNueva({ rfc: '', razon_social: '' });
    } catch (e) {
      setError(String(e));
    }
  };

  return (
    <div className="flex items-center gap-2 text-sm">
      <Campo
        id="empresa-selector"
        label="Empresa activa"
        hint="Empresa cuyos datos se muestran en todas las pantallas."
        className="min-w-52"
      >
        <select
          className="border rounded p-1 w-full"
          value={activaId ?? ''}
          onChange={(e) => cambiar(Number(e.target.value))}
        >
          {empresas.map((e) => (
            <option key={e.id ?? 0} value={e.id ?? 0}>
              {e.razon_social} ({e.rfc})
            </option>
          ))}
        </select>
      </Campo>
      <button
        type="button"
        className="border rounded px-2 py-1"
        onClick={() => setMostrarAlta((v) => !v)}
      >
        + Nueva
      </button>
      {mostrarAlta && (
        <span className="flex items-start gap-2">
          <Campo
            id="nueva-empresa-rfc"
            label="RFC de la nueva empresa"
            hint="13 caracteres. Ejemplo: XAXX010101000."
            className="min-w-40"
          >
            <input
              className="border rounded p-1 w-full"
              placeholder="XAXX010101000"
              value={nueva.rfc}
              onChange={(e) => setNueva({ ...nueva, rfc: e.target.value.toUpperCase() })}
            />
          </Campo>
          <Campo
            id="nueva-empresa-razon-social"
            label="Razón social de la nueva empresa"
            hint="Nombre legal del empleador. Ejemplo: Mi Empresa S.A. de C.V."
            className="min-w-52"
          >
            <input
              className="border rounded p-1 w-full"
              placeholder="Mi Empresa S.A. de C.V."
              value={nueva.razon_social}
              onChange={(e) => setNueva({ ...nueva, razon_social: e.target.value })}
            />
          </Campo>
          <button
            type="button"
            className="bg-blue-600 text-white rounded px-2 py-1 mt-6"
            onClick={crear}
          >
            Guardar
          </button>
        </span>
      )}
      {error && <span className="text-red-600">{error}</span>}
    </div>
  );
}
