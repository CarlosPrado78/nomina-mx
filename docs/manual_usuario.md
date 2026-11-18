# Manual de usuario — NominaMX v0.9 (piloto)

Este manual es para la persona que prepara la nómina: no necesita saber
programación. La aplicación guarda toda la información en su propia computadora;
no envía datos a Internet para calcular.

## 1. Instalación

### macOS

1. Descargue `NominaMX_0.1.0_aarch64.dmg` desde el artefacto
   `NominaMX-macos-latest` del run de GitHub Actions indicado por soporte.
2. Abra el `.dmg` y arrastre **NominaMX.app** a la carpeta **Aplicaciones**.
3. La primera vez, macOS puede impedir la apertura porque la aplicación aún no
   tiene firma de código. Haga clic derecho sobre la app y elija **Abrir**, o
   vaya a **Preferencias del Sistema → Seguridad y Privacidad → Abrir de todas
   formas**. Solo se hace una vez.
4. Abra NominaMX desde Aplicaciones.

### Windows

1. Descargue `NominaMX_0.1.0_x64_en-US.msi` desde el artefacto
   `NominaMX-windows-latest` del run indicado por soporte.
2. Haga doble clic y siga el asistente de instalación.
3. SmartScreen mostrará una advertencia por falta de firma: haga clic en
   **Más información → Ejecutar de todas formas**. Solo se hace una vez.
4. Abra NominaMX desde el menú Inicio.

Si la aplicación no abre, reinicie la computadora e intente de nuevo. Si vuelve
a fallar, reporte el error con una captura (consulte el apartado de soporte en
`docs/release_notes_v0_9.md`, cuando exista).

## 2. Configuración inicial de la empresa

En el menú superior entre a **Empresa** y llene:

| Campo | Qué poner |
|-------|-----------|
| RFC | RFC de la persona moral o física que paga la nómina |
| Razón social | Nombre completo del empleador |
| Régimen fiscal | Código del régimen, por ejemplo `601` |
| CP | Código postal del domicilio fiscal |
| Estado ISN | Entidad federativa donde se causa el ISN |
| Tasa ISN % | Tasa que corresponda a ese estado (ej. 3.00) |
| Prima riesgo trabajo % | Prima de la empresa según su clase de riesgo |
| Registro patronal IMSS | Número de registro patronal |

Oprima **Guardar**. Si va a manejar más de una empresa, use el selector de
empresa en la parte superior para cambiar de una a otra; cada empresa tiene sus
propios trabajadores, periodos y recibos.

## 3. Alta de trabajador

En **Trabajadores** llene el formulario. Los campos más importantes son:

| Campo | Qué significa |
|-------|---------------|
| Nombre | Nombre completo, tal como aparece en su identificación |
| CURP | CURP de 18 caracteres |
| RFC | RFC del trabajador |
| NSS | Número de seguro social |
| Número de empleado | Clave interna, si la usa |
| Fecha de nacimiento | Se usa para datos del recibo |
| Estado civil | Estado civil del trabajador |
| Fecha de ingreso | Día en que empezó a laborar; sirve para antigüedad y vacaciones |
| Tipo contrato | Por ejemplo `indefinido` o `temporal` |
| Departamento / Puesto | Datos informativos para el recibo |
| Salario diario | Salario de un día, con dos decimales |
| Periodicidad | Semanal, quincenal o mensual |
| Jornada | Diurna, nocturna o mixta |
| Horas semanales | Normalmente 40 |
| INFONAVIT activo | Marque si el trabajador tiene crédito vigente |
| Número de crédito INFONAVIT | Lo emite el INFONAVIT |
| Factor INFONAVIT | Según su tipo de descuento; si no sabe, deje 1.0 o consulte su recibo de SUA |

Guarde con el botón **Guardar**. Una vez dada de alta, aparecerá en la lista de
trabajadores al calcular la nómina.

## 4. Captura del periodo y cálculo del recibo

En **Nomina**:

1. Elija la periodicidad: Semanal, Quincenal o Mensual.
2. Capture la fecha de inicio, la fecha de fin y la **fecha de pago**. Esta última
   es muy importante: la aplicación busca las tablas fiscales vigentes en esa
   fecha (UMA, subsidio e ISR).
3. Escriba el número de días del periodo.
4. Si hubo tiempo extraordinario, capture las horas dobles/triples, las semanas
   y la jornada. Si el trabajador tiene salario mínimo, marque la casilla
   correspondiente para aplicar la regla del artículo 93 fracción I de la LISR.
5. Oprima **Crear periodo**. Verá el número de periodo asignado.
6. Marque la casilla de cada trabajador que deba aparecer en esa nómina.
7. Oprima **Calcular nomina**. Aparecerá una tabla con percepciones, ISR, IMSS,
   INFONAVIT, neto y costo patronal por trabajador.

Revise siempre el neto contra su cálculo de SUA antes de pagar. El ISR usa la
tabla vigente a la fecha de pago; por eso un periodo de enero 2026 utiliza la
UMA y el subsidio de enero, aunque lo calcule en noviembre.

### PDF del recibo

La aplicación cuenta con el generador de PDF con acentos (`José Pérez Núñez`),
pero en esta versión **el botón de descarga aún no está enlazado en la pantalla
de nómina**. Mientras tanto, el PDF de evidencia se encuentra en
`docs/evidencia/recibo_jose_perez.pdf`. Este punto se reporta como bloqueante
específico en `docs/smoke_test.md`.

## 5. Importación de trabajadores desde CSV

Use **Importar CSV** cuando tenga muchos trabajadores nuevos y ya capturó la
plantilla. No lo use para cambiar el salario de uno solo: para eso edite el
trabajador directamente.

1. Descargue la plantilla con **Descargar plantilla**.
2. Llénela en Excel o en un editor de texto, guardando como CSV UTF-8.
3. Seleccione el archivo y oprima **Importar**.

La aplicación valida cada fila por separado: si una está mal, muestra el número
de fila y el motivo, pero continúa con las demás. Los trabajadores se asignan a
la empresa activa automáticamente.

## 6. Finiquitos y liquidaciones

En **Finiquitos y liquidaciones** elija:

- **Finiquito (renuncia):** cuando el trabajador deja la empresa por su propia
  decisión o por terminación de contrato. Incluye días pendientes de pago,
  vacaciones no gozadas y la parte proporcional de aguinaldo.
- **Liquidación (despido injustificado):** cuando la empresa termina la
  relación sin causa justificada. Además del finiquito, incluye indemnización
  constitucional (90 días por año de antigüedad) y prima de antigüedad.

Capture salario diario, días pendientes, días de vacaciones pendientes, días
trabajados en el año y años de antigüedad, y oprima **Calcular**.

## 7. Recálculo retroactivo

Use **Recálculo** cuando haya cambiado un salario o una prestación después de
cerrar un periodo y necesite corregir los recibos.

- Seleccione el periodo y escriba un **motivo**; no se permite dejarlo vacío.
- Puede recalcular todo el periodo o solo un trabajador.
- La aplicación usa siempre los parámetros vigentes a la **fecha de pago
  original**. Un periodo de enero 2026 usa UMA de $113.14 y subsidio de $536.21,
  aunque hoy sea noviembre.
- Muestra una tabla con percepciones, deducciones y neto antes y después, con su
  diferencia.

**Advertencias:**

- Los recibos que ya tienen CFDI timbrado no se modifican: solo se reporta la
  diferencia. El XML timbrado queda intacto.
- El CFDI de ajuste (`TipoDeComprobante=E`) está pendiente para v1.1; esta
  versión solo calcula y reporta.
- Si el recibo original tenía horas extra, se reconstruyen como jornada diurna
  y una semana. Revise el XML original si el caso fue distinto.

## 8. Respaldo y restauración

La base de datos se guarda en:

- macOS: `~/Library/Application Support/com.nomina.mx/nomina.db`
- Windows: `%APPDATA%\com.nomina.mx\nomina.db`

Haga una copia de ese archivo al terminar cada nómina y guárdela en otra unidad.
En esta versión no hay un botón visible de respaldo, pero el motor sí cuenta con
los comandos `backup_db`, `listar_backups` y `restore_db`: crean copias con
nombre `nomina_backup_AAAAMMDD_HHMMSS.db`, nunca sobrescriben una existente y,
antes de restaurar, validan que el archivo sea una base de datos íntegra y
guardan una copia del estado actual. Solicite a soporte el procedimiento si
necesita restaurar.

Guarde también los XML de los CFDI y los PDF de los recibos: la base de datos no
sustituye a los comprobantes.

## 9. Preguntas frecuentes

**¿La aplicación necesita Internet?** No para calcular. Solo se usa Internet para
descargar la instalación inicial y, cuando se implemente, para timbrar.

**¿Por qué el ISR de enero es distinto al de febrero?** Porque el subsidio al
empleo cambió de $536.21 a $535.65 el 1 de febrero de 2026. Se usa la tabla
vigente en la fecha de pago.

**¿Puedo tener dos empresas?** Sí. Use el selector de empresa en el encabezado.
Los datos de una no se mezclan con los de la otra.

**¿Qué hago si el neto no coincide con SUA?** Revise el salario diario, la prima
de riesgo de trabajo, el factor de INFONAVIT y las fechas del periodo. Si sigue
diferente, use los casos de `docs/casos_validacion_contable.md` para ubicar el
concepto y repórtelo.

**¿Se puede timbrar desde esta versión?** No. El timbrado real con CSD y PAC está
pendiente. Los comprobantes generados son de muestra y no tienen validez fiscal.

**¿Qué hago si la app no abre en Windows por SmartScreen?** Clic en **Más
información → Ejecutar de todas formas**. Es una advertencia por falta de firma
de código, no un virus.




