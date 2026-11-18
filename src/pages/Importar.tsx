import { useState } from 'react';
import { open } from '@tauri-apps/plugin-dialog';
import { api, type ImportResult } from '../lib/tauri';

/**
 * C2 — Importación masiva de trabajadores.
 *
 * El archivo se lee en Rust (UTF-8 con fallback Latin-1). La UI únicamente
 * entrega la ruta absoluta; `empresa_id` nunca viaja desde el CSV.
 */
const PLANTILLA = [
  'nombre,curp,rfc,nss,fecha_ingreso,salario_diario,periodicidad_pago,tipo_contrato,num_empleado,departamento,puesto,infonavit_activo,infonavit_factor,infonavit_tipo',
  'Ana Rodríguez,RODR900101HDFNSR08,RODR900101AB1,12345678901,2020-01-15,450.50,mensual,indefinido,101,Producción,Operadora,0,0,CUOTA_FIJA_VSM',
].join('\n');

const REQUERIDAS = [
  'nombre', 'curp', 'rfc', 'nss', 'fecha_ingreso',
  'salario_diario', 'periodicidad_pago', 'tipo_contrato',
];

const CATALOGOS = {
  periodicidad_pago: 'diaria, semanal, quincenal, mensual, bimestral, trimestral, semestral, anual',
  tipo_contrato: '01/indefinido, 02/temporal, 03/obra_determinada',
  infonavit_activo: '1/0, true/false o si/no',
};

export default function ImportarPage() {
  const [ruta, setRuta] = useState<string | null>(null);
  const [resultado, setResultado] = useState<ImportResult | null>(null);
  const [error, setError] = useState('');
  const [importando, setImportando] = useState(false);

  const seleccionar = async () => {
    setError('');
    try {
      const seleccionada = await open({
        multiple: false,
        directory: false,
        filters: [{ name: 'CSV', extensions: ['csv'] }],
      });
      if (typeof seleccionada === 'string') {
        setRuta(seleccionada);
        setResultado(null);
      }
    } catch (e) {
      setError(`No se pudo abrir el selector de archivos: ${String(e)}`);
    }
  };

  const importar = async () => {
    if (!ruta) return;
    setImportando(true);
    setError('');
    setResultado(null);
    try {
      setResultado(await api.importarTrabajadoresCsv(ruta));
    } catch (e) {
      setError(String(e));
    } finally {
      setImportando(false);
    }
  };

  const descargarPlantilla = () => {
    // BOM UTF-8 para que Excel en español respete los acentos.
    const blob = new Blob([`\uFEFF${PLANTILLA}\n`], {
      type: 'text/csv;charset=utf-8',
    });
    const url = URL.createObjectURL(blob);
    const enlace = document.createElement('a');
    enlace.href = url;
    enlace.download = 'trabajadores_template.csv';
    document.body.appendChild(enlace);
    enlace.click();
    document.body.removeChild(enlace);
    setTimeout(() => URL.revokeObjectURL(url), 0);
  };

  return (
    <div className="max-w-4xl">
      <h1 className="text-xl font-bold mb-1">Importación masiva de trabajadores</h1>
      <p className="text-sm text-gray-600 mb-4">
        CSV en UTF-8 (o Latin-1 desde Excel). Cada fila se valida y se inserta en su propia
        transacción: una fila inválida no detiene el archivo. El <code>empresa_id</code> se asigna
        automáticamente desde la empresa activa.
      </p>

      <div className="flex flex-wrap gap-2 mb-4">
        <button
          type="button"
          onClick={seleccionar}
          className="border rounded px-3 py-2 bg-white hover:bg-gray-50"
        >
          Seleccionar archivo CSV
        </button>
        <button
          type="button"
          onClick={importar}
          disabled={!ruta || importando}
          className="rounded px-3 py-2 bg-blue-600 text-white disabled:opacity-50"
        >
          {importando ? 'Importando…' : 'Importar'}
        </button>
        <button
          type="button"
          onClick={descargarPlantilla}
          className="rounded px-3 py-2 bg-gray-800 text-white"
        >
          Descargar plantilla
        </button>
      </div>

      {ruta && (
        <p className="text-sm mb-4 break-all">
          <span className="font-semibold">Archivo:</span> {ruta}
        </p>
      )}
      {error && (
        <div className="border border-red-300 bg-red-50 text-red-800 rounded p-3 mb-4 whitespace-pre-wrap">
          {error}
        </div>
      )}

      {resultado && (
        <div className="space-y-4">
          <div className="border rounded p-3 bg-green-50">
            <span className="font-semibold">Insertados:</span> {resultado.insertados}
            {' · '}
            <span className="font-semibold">Errores de fila:</span> {resultado.errores.length}
          </div>
          {resultado.errores.length > 0 && (
            <table className="w-full text-sm border-collapse">
              <thead>
                <tr className="bg-gray-100 text-left">
                  <th className="border p-2">Fila</th>
                  <th className="border p-2">Columna</th>
                  <th className="border p-2">Mensaje</th>
                </tr>
              </thead>
              <tbody>
                {resultado.errores.map((e, i) => (
                  <tr key={`${e.fila}-${e.columna}-${i}`}>
                    <td className="border p-2">{e.fila}</td>
                    <td className="border p-2">{e.columna}</td>
                    <td className="border p-2">{e.mensaje}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          )}
        </div>
      )}

      <details className="mt-6 text-sm text-gray-700">
        <summary className="cursor-pointer font-semibold">Formatos y catálogos</summary>
        <div className="mt-2 space-y-2">
          <p>
            <span className="font-semibold">Columnas requeridas:</span>{' '}
            {REQUERIDAS.join(', ')}.
          </p>
          <p>
            <span className="font-semibold">Opcionales:</span> num_empleado, departamento,
            puesto, infonavit_activo, infonavit_factor, infonavit_tipo.
          </p>
          {Object.entries(CATALOGOS).map(([clave, valores]) => (
            <p key={clave}>
              <span className="font-semibold">{clave}:</span> {valores}.
            </p>
          ))}
          <p>
            CURP 18 caracteres, RFC 12–13 caracteres, NSS 11 dígitos y fecha{' '}
            <code>AAAA-MM-DD</code>. La fila 1 del CSV es el encabezado; los errores reportan
            el número de fila real para corregir el archivo.
          </p>
        </div>
      </details>
    </div>
  );
}
