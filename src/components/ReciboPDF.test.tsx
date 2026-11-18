import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { renderToStaticMarkup } from 'react-dom/server';
import { BotonDescargarRecibo, ReciboDocument, descargarRecibo } from './ReciboPDF';

// Capturamos el elemento que ReciboPDF envía a @react-pdf/renderer sin
// reemplazar el renderizador real (StyleSheet, Document, Page, Text, View).
const { pdfMock } = vi.hoisted(() => ({
  pdfMock: vi.fn((_elemento: any) => ({
    toBlob: vi.fn(async () => new Blob(['%PDF-1.4'], { type: 'application/pdf' })),
  })),
}));

vi.mock('@react-pdf/renderer', async (importOriginal) => {
  const actual = await importOriginal<any>();
  return { ...actual, pdf: pdfMock };
});

const RECIBO = {
  trabajador_id: 7,
  trabajador_nombre: 'José Pérez Núñez',
  total_percepciones: 10000,
  isr: 247.21,
  imss_obrero: 265.11,
  infonavit_desc: 0,
  neto: 9487.68,
  costo_total_patron: 3288.26,
};

function textoDe(nodo: any): string {
  if (nodo === null || nodo === undefined || typeof nodo === 'boolean') return '';
  if (typeof nodo === 'string' || typeof nodo === 'number') return String(nodo);
  if (Array.isArray(nodo)) return nodo.map(textoDe).join(' ');
  if (typeof nodo === 'object' && 'props' in nodo) {
    // El elemento pasado a pdf() es <ReciboDocument/>, no su resultado: hay
    // que invocarlo una vez para llegar a los <Text> con el nombre acentuado.
    if (nodo.type === ReciboDocument) return textoDe(ReciboDocument(nodo.props));
    return textoDe(nodo.props?.children);
  }
  return '';
}

describe('BotonDescargarRecibo', () => {
  it('existe en el HTML y llama a la función de descarga al hacer click', () => {
    const onDescargar = vi.fn();
    const elemento = BotonDescargarRecibo({ recibo: RECIBO, onDescargar });

    expect(elemento.type).toBe('button');
    expect(renderToStaticMarkup(<BotonDescargarRecibo recibo={RECIBO} onDescargar={onDescargar} />))
      .toContain('Descargar PDF');

    (elemento.props as any).onClick();
    expect(onDescargar).toHaveBeenCalledTimes(1);
    expect(onDescargar).toHaveBeenCalledWith(RECIBO);
  });

  it('usa descargarRecibo por defecto cuando no se inyecta callback', () => {
    const html = renderToStaticMarkup(<BotonDescargarRecibo recibo={RECIBO} />);
    expect(html).toContain('Descargar PDF');
    expect(html).toContain('Descargar PDF del recibo 7');
  });
});

describe('descargarRecibo', () => {
  let ancla: { href: string; download: string; click: ReturnType<typeof vi.fn> };

  beforeEach(() => {
    pdfMock.mockClear();
    ancla = { href: '', download: '', click: vi.fn() };
    vi.stubGlobal('document', { createElement: vi.fn(() => ancla) });
    vi.stubGlobal('URL', {
      createObjectURL: vi.fn(() => 'blob:recibo'),
      revokeObjectURL: vi.fn(),
    });
  });

  afterEach(() => {
    vi.unstubAllGlobals();
  });

  it('genera el PDF con el nombre acentuado y descarga el archivo', async () => {
    await descargarRecibo(RECIBO);

    expect(pdfMock).toHaveBeenCalledTimes(1);
    const documento = pdfMock.mock.calls[0][0];
    expect(textoDe(documento)).toContain('José Pérez Núñez');
    expect(textoDe(ReciboDocument({ recibo: RECIBO }))).toContain('José Pérez Núñez');

    expect(ancla.download).toBe('recibo_7.pdf');
    expect(ancla.href).toBe('blob:recibo');
    expect(ancla.click).toHaveBeenCalledTimes(1);
    expect(URL.revokeObjectURL).toHaveBeenCalledWith('blob:recibo');
  });
});
