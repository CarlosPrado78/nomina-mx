import { describe, expect, it, vi } from 'vitest';
import { renderToStaticMarkup } from 'react-dom/server';
import { MemoryRouter } from 'react-router-dom';

// Las pantallas solo llaman a la API dentro de useEffect, que no se ejecuta en
// renderizado estático. El mock evita depender de Tauri en el entorno de tests.
vi.mock('./lib/tauri', () => ({
  api: new Proxy({}, {
    get: (_objeto, metodo) => vi.fn(async () => {
      if (metodo === 'listarEmpresas' || metodo === 'listarTrabajadores' ||
          metodo === 'listarPeriodos') return [];
      return null;
    }),
  }),
}));
vi.mock('@tauri-apps/plugin-dialog', () => ({ open: vi.fn() }));

import App from './App';
import AuditoriaPage from './pages/Auditoria';
import EmpresaPage from './pages/Empresa';
import EmpresaSelector from './pages/EmpresaSelector';
import FiniquitosPage from './pages/Finiquitos';
import ImportarPage from './pages/Importar';
import NominaPage from './pages/Nomina';
import RecalculoPage from './pages/Recalculo';
import TrabajadoresPage from './pages/Trabajadores';

interface Control {
  tag: string;
  id?: string;
  type?: string;
  placeholder?: string;
  'aria-describedby'?: string;
}

/** Extrae los atributos de una etiqueta HTML renderizada. */
function atributos(etiqueta: string): Record<string, string> {
  const resultado: Record<string, string> = {};
  for (const coincidencia of etiqueta.matchAll(/([a-zA-Z-]+)="([^"]*)"/g)) {
    resultado[coincidencia[1]] = coincidencia[2];
  }
  return resultado;
}

/** Controles (input/select/textarea) presentes en el HTML estático. */
function controlesDe(html: string): Control[] {
  return [...html.matchAll(/<(input|select|textarea)\b[^>]*>/g)].map((coincidencia) => ({
    tag: coincidencia[1],
    ...atributos(coincidencia[0]),
  })) as Control[];
}

/** Los tipos que muestran un placeholder de ejemplo al usuario. */
function llevaPlaceholder(control: Control): boolean {
  if (control.tag === 'textarea') return true;
  if (control.tag !== 'input') return false;
  if (!control.type) return true;
  return ['text', 'number', 'search', 'email', 'tel', 'url', 'password'].includes(control.type);
}

// Importar.tsx no aparece aquí: auditeda, no tiene campos de captura. El archivo
// se elige con el diálogo nativo del sistema operativo, no con un <input>.
const PANTALLAS: Array<[string, () => JSX.Element]> = [
  ['Empresa', EmpresaPage],
  ['Trabajadores', TrabajadoresPage],
  ['Nomina', NominaPage],
  ['Finiquitos', FiniquitosPage],
  ['Recalculo', RecalculoPage],
  ['Auditoria', AuditoriaPage],
  ['EmpresaSelector', EmpresaSelector],
];

describe('Etiquetas claras en todos los formularios', () => {
  it.each(PANTALLAS)('%s: cada control tiene label, ayuda y placeholder', (_nombre, Pagina) => {
    const html = renderToStaticMarkup(<Pagina />);
    // Los campos ocultos no se muestran al usuario: no requieren label ni ayuda.
    const controles = controlesDe(html).filter((c) => c.type !== 'hidden');
    expect(controles.length, 'la pantalla debe tener controles').toBeGreaterThan(0);

    const idsConLabel = new Set(
      [...html.matchAll(/<label\b[^>]*\bfor="([^"]+)"/g)].map((m) => m[1]),
    );
    const idsExistentes = new Set(
      [...html.matchAll(/\bid="([^"]+)"/g)].map((m) => m[1]),
    );

    for (const control of controles) {
      const nombre = `<${control.tag} id="${control.id ?? ''}">`;
      expect(control.id, `${nombre} debe tener id`).toBeTruthy();
      expect(idsConLabel.has(control.id!), `${nombre} debe tener <label for>`).toBe(true);
      expect(control['aria-describedby'], `${nombre} debe tener aria-describedby`).toBeTruthy();
      expect(
        idsExistentes.has(control['aria-describedby']!),
        `${nombre} debe apuntar a un texto de ayuda existente`,
      ).toBe(true);
      if (llevaPlaceholder(control)) {
        expect(control.placeholder, `${nombre} debe tener placeholder de ejemplo`).toBeTruthy();
      }
    }
  });

  it('Importar CSV no tiene campos sin etiquetar', () => {
    const html = renderToStaticMarkup(<ImportarPage />);
    // El archivo se elige en un diálogo nativo; no hay inputs que etiquetar.
    expect(controlesDe(html)).toHaveLength(0);
    expect(html).toContain('Seleccionar archivo CSV');
    expect(html).toContain('AAAA-MM-DD');
  });

  it('los campos sensibles muestran el ejemplo y la ayuda pedidos', () => {
    const empresa = renderToStaticMarkup(<EmpresaPage />);
    expect(empresa).toContain('RFC de la empresa');
    expect(empresa).toContain('placeholder="XAXX010101000"');
    expect(empresa).toContain('Razón social');
    expect(empresa).toContain('placeholder="Mi Empresa S.A. de C.V."');
    expect(empresa).toContain('Régimen fiscal (clave SAT)');
    expect(empresa).toContain('601 = General de Ley Personas Morales, 605 = Sueldos y Salarios');
    expect(empresa).toContain('Estado de la República');
    expect(empresa).toContain('Prima de riesgo de trabajo (%)');
    expect(empresa).toContain('placeholder="0.5"');
    expect(empresa).toContain('Registro patronal IMSS');
    expect(empresa).toContain('placeholder="A1234567890"');

    const trabajador = renderToStaticMarkup(<TrabajadoresPage />);
    expect(trabajador).toContain('placeholder="PENJ850101HDFRRN01"');
    expect(trabajador).toContain('placeholder="PENJ850101AB1"');
    expect(trabajador).toContain('placeholder="12345678901"');
    expect(trabajador).toContain('Fecha de nacimiento');
    expect(trabajador).toContain('Fecha de ingreso a la empresa');
    expect(trabajador).toContain('Formato AAAA-MM-DD.');
    expect(trabajador).toContain('Salario diario (pesos)');
    expect(trabajador).toContain('Horas semanales');
    expect(trabajador).toContain('placeholder="48"');

    const nomina = renderToStaticMarkup(<NominaPage />);
    expect(nomina).toContain('Fecha de inicio del periodo');
    expect(nomina).toContain('Fecha de fin del periodo');
    expect(nomina).toContain('Fecha de pago');
    expect(nomina).toContain('Días pagados');
    expect(nomina).toContain('Trabajadores a incluir');
  });
});

describe('Crédito del desarrollador', () => {
  it('aparece en el header bajo el nombre de la app', () => {
    const html = renderToStaticMarkup(
      <MemoryRouter initialEntries={['/']}>
        <App />
      </MemoryRouter>,
    );
    expect(html).toContain('Developed by Carlos G. Prado Cabrera');
    expect(html).toContain('class="text-xs text-muted-foreground"');
    expect(html).toContain('NominaMX');
  });
});
