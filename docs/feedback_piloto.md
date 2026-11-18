# Plan de feedback — Piloto v0.9 con despachos contables

## Objetivo

Validar que NominaMX produzca cifras congruentes con SUA y que el flujo de
captura sea comprensible para un contador, antes de invertir en firma de código,
PAC real y lanzamiento público.

## 1. Formulario de feedback

Se recomienda un formulario de Google Forms (o Microsoft Forms) con este
contenido. El enlace se enviará por correo a cada despacho participante.

### Datos generales

- Nombre del despacho:
- Nombre de quien capturó la nómina:
- Correo de contacto:
- Sistema operativo usado (macOS / Windows):
- Fecha de la captura:

### Cierre de la nómina

- Número de trabajadores procesados:
- Tiempo total desde que abrió la app hasta que terminó el cálculo (minutos):
- Tiempo que llevaba la misma nómina con el proceso anterior (minutos):
- ¿Pudo crear empresa, trabajadores, periodo y recibo sin ayuda? (Sí / No / Parcial)
- ¿En qué paso se detuvo o dudó? (campo abierto)

### Cifras

- ¿Comparó el resultado contra SUA? (Sí / No)
- Si comparó: ¿cuántos recibos quedaron exactamente igual?
- Si hubo diferencias: concepto y monto de cada una:
- ¿Los acentos del PDF se ven correctamente? (Sí / No / No se generó PDF)

### Incidentes

- ¿La app se cerró o mostró un error? (Sí / No)
- Describa el error con sus palabras y, si puede, adjunte captura:
- ¿Pudo recuperar su trabajo? (Sí / No)

### Features faltantes

- ¿Qué le gustaría que agreguemos primero? (opción múltiple)
  - Botón de descarga de PDF en la pantalla de recibo
  - Timbrado real con PAC
  - Edición de trabajador desde la lista
  - Reporte comparativo contra SUA
  - Otro (campo abierto)
- Comentarios libres:

## 2. Métricas a recolectar

| Métrica | Cómo se mide | Meta |
|---------|--------------|------|
| Tiempo por nómina | Tiempo declarado en el formulario, comparado con el proceso anterior | Igual o menor al proceso actual |
| Errores de cálculo | Número de recibos con diferencia contra SUA | 0 diferencias sin explicación documentada |
| Errores de la app | Número de incidentes reportados | 0 incidentes que impidan terminar la nómina |
| Finalización del flujo | Porcentaje de despachos que crearon empresa → trabajador → periodo → recibo | 100% |
| Aceptación de la interfaz | Respuesta 'Sí' en la pregunta '¿pudo … sin ayuda?' | Al menos 2 de 3 despachos |
| Features faltantes | Votos por opción en la encuesta | Ordenar el backlog para v1.0 |
| Acentos en PDF | Respuesta sobre 'José Pérez Núñez' en Preview/Edge | 100% correcto |

Cada métrica se registra en una hoja de cálculo simple con una fila por
despacho y una columna por métrica. No se recopilan CURP, RFC ni salarios
reales en el formulario: solo números agregados.

## 3. Cronograma

| Semana | Actividad |
|--------|-----------|
| 1 | Entrega del instalador, del manual de usuario y del formulario. Cada despacho procesa al menos una nómina real o de prueba. |
| 2 | El despacho completa el formulario y adjunta sus capturas. Se cierran los casos de validación contable de `docs/casos_validacion_contable.md`. |
| Final de la semana 2 | Reunión de revisión de 45 minutos con los 3 despachos: diferencias contra SUA, errores, features faltantes y prioridades. |

### Entregable de la reunión

- Acta con los acuerdos y responsables.
- Lista priorizada de correcciones para v1.0.
- Decisión: iniciar firma de código y timbrado real, o extender el piloto dos semanas más.

## 4. Reglas de privacidad

- Los datos de nómina se quedan en la computadora del despacho; no se suben a ningún servidor.
- El formulario solo recopila opiniones y tiempos, nunca información personal de los trabajadores.
- Las capturas que se compartan deben tener los RFC y NSS cubiertos.

## 5. Contacto

El canal de reporte se indicará en el correo de entrega del piloto. Al reportar,
incluya sistema operativo, versión de la app, pasos para reproducir y captura de
pantalla.

