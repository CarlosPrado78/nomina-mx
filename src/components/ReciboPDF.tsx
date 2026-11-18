import { Document, Page, Text, View, StyleSheet, pdf } from '@react-pdf/renderer';

const styles = StyleSheet.create({
  page: { padding: 24, fontSize: 10 },
  title: { fontSize: 14, marginBottom: 8, fontWeight: 'bold' },
  row: { flexDirection: 'row', justifyContent: 'space-between', marginBottom: 2 },
});

export function ReciboDocument({ recibo }: { recibo: any }) {
  return (
    <Document>
      <Page style={styles.page}>
        <Text style={styles.title}>Recibo de Nomina</Text>
        <View style={styles.row}><Text>Trabajador</Text><Text>{recibo.trabajador_nombre || `ID ${recibo.trabajador_id}`}</Text></View>
        <View style={styles.row}><Text>Percepciones</Text><Text>{recibo.total_percepciones?.toFixed(2)}</Text></View>
        <View style={styles.row}><Text>ISR</Text><Text>{recibo.isr?.toFixed(2)}</Text></View>
        <View style={styles.row}><Text>IMSS obrero</Text><Text>{recibo.imss_obrero?.toFixed(2)}</Text></View>
        <View style={styles.row}><Text>INFONAVIT</Text><Text>{recibo.infonavit_desc?.toFixed(2)}</Text></View>
        <View style={styles.row}><Text>Neto</Text><Text>{recibo.neto?.toFixed(2)}</Text></View>
        <View style={styles.row}><Text>Costo patronal</Text><Text>{recibo.costo_total_patron?.toFixed(2)}</Text></View>
      </Page>
    </Document>
  );
}

export async function descargarRecibo(recibo: any) {
  const blob = await pdf(<ReciboDocument recibo={recibo} />).toBlob();
  const url = URL.createObjectURL(blob);
  const a = document.createElement('a');
  a.href = url;
  a.download = `recibo_${recibo.trabajador_id}.pdf`;
  a.click();
  URL.revokeObjectURL(url);
}

// Botón reutilizable en la tabla de recibos calculados.
// `onDescargar` es inyectable para poder probar el click sin DOM real.
export function BotonDescargarRecibo({
  recibo,
  onDescargar = descargarRecibo,
}: {
  recibo: any;
  onDescargar?: (recibo: any) => void | Promise<void>;
}) {
  return (
    <button
      type="button"
      onClick={() => onDescargar(recibo)}
      className="bg-emerald-600 text-white px-2 py-1 rounded text-xs"
      aria-label={`Descargar PDF del recibo ${recibo.trabajador_id}`}
    >
      Descargar PDF
    </button>
  );
}