# Manual de usuario — NominaMX v0.9 (piloto)

Este manual es para la persona que prepara la nómina: no necesita saber
programación. La aplicación guarda toda la información en su propia computadora;
no envía datos a Internet para calcular.

## 1. Instalación

### macOS

1. Compruebe la arquitectura en el menú Apple → **Acerca de esta Mac** (o con
   `uname -m`). Descargue `NominaMX_0.1.0_aarch64.dmg` desde el artefacto
   `NominaMX-macos-arm64` si es Apple Silicon, o `NominaMX_0.1.0_x64.dmg` desde
   `NominaMX-macos-x86_64` si es Intel, del run de GitHub Actions indicado por
   soporte.
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

## 1.1 Verificación post-instalación (usuario nuevo en macOS)

Si quiere repetir la comprobación en una cuenta de macOS sin las herramientas
del desarrollador, siga estos pasos:

1. Cree un usuario estándar (no administrador) llamado
   `VerificacionNominaMX` desde **Preferencias del Sistema → Usuarios y grupos**.
   No instale Homebrew, Python, Node ni Rust en esa cuenta.
2. Inicie sesión con ese usuario y abra **Terminal**.
3. Desde la carpeta del proyecto ejecute:

   ```bash
   GH_TOKEN=su_token scripts/verificacion_macos.sh
   ```

   El script descarga el `.dmg` del último run verde, lo monta, copia la app a
   Aplicaciones, la abre, espera cinco segundos, toma la captura
   `macos_01_app.png` y comprueba que se creó
   `~/Library/Application Support/com.nomina.mx/nomina.db`. Todo queda
   registrado en `docs/evidencia/verificacion_visual/macos/reporte.md`.

4. **Importante:** el script no crea empresa ni trabajador. Con la app abierta,
   hágalo manualmente: empresa con RFC `XAXX010101000` y régimen `601`;
   trabajador José Pérez Núñez con CURP `PENJ850101HDFRRN01`, RFC
   `PENJ850101AB1`, NSS `12345678901`, ingreso `2026-01-15`, salario diario
   `500.00`, periodicidad quincenal y contrato indeterminado. Después cree el
   periodo, calcule el recibo y exporte el PDF.
5. Tome las capturas `macos_02_trabajador.png`, `macos_03_recibo.png`,
   `macos_04_pdf_acentos.png` y `macos_05_bd_ruta.png` en la misma carpeta.
   Cierre la app cuando termine; el script no la cierra por usted.

## 1.2 Cómo leer los formularios

Todos los formularios muestran, debajo de cada campo, una **etiqueta en español**,
un **ejemplo en el cuadro** (placeholder) y una **ayuda de una línea**. La ayuda
indica qué dato se espera, el formato de las fechas (`AAAA-MM-DD`) o la unidad de
los números (pesos, días, horas, porcentaje). Los catálogos (estado, régimen
fiscal, jornada, etc.) son listas desplegables: no se escriben a mano.

En la esquina superior derecha del encabezado aparece **NominaMX** y, justo
debajo, el crédito *Developed by Carlos G. Prado Cabrera*. Junto a él está el
selector **Empresa activa**: cambiar de empresa recarga todas las pantallas con
los datos de esa empresa. El botón **+ Nueva** permite dar de alta otra empresa;
sus campos **RFC de la nueva empresa** y **Razón social de la nueva empresa**
también muestran ejemplo y ayuda.

## 2. Configuración inicial de la empresa

En el menú superior entre a **Empresa** y llene. La columna *Ejemplo* es el texto
que aparece dentro del cuadro; la *Ayuda* aparece debajo de él:

| Etiqueta del campo | Ejemplo | Ayuda |
|--------------------|---------|-------|
| RFC de la empresa | `XAXX010101000` | 13 caracteres para persona moral. Para pruebas use `XAXX010101000`. |
| Razón social | `Mi Empresa S.A. de C.V.` | Nombre legal, tal como aparece en su constancia de situación fiscal. |
| Régimen fiscal (clave SAT) | `601` | 601 = General de Ley Personas Morales, 605 = Sueldos y Salarios, 612 = Personas Físicas con Actividades Empresariales. Se elige de una lista. |
| Código postal | `01000` | 5 dígitos del domicilio fiscal. |
| Estado de la República | `CIUDAD DE MEXICO` | Entidad federativa donde se calcula el ISN. Lista desplegable, no texto libre. |
| Tasa de ISN (%) | `3` | Porcentaje del impuesto sobre nóminas de esa entidad. |
| Prima de riesgo de trabajo (%) | `0.5` | Prima anual; oscila entre 0.5% (mínima) y 15% (máxima). Si es la primera vez, usar 0.5%. |
| Registro patronal IMSS | `A1234567890` | Opcional. Si no lo tienes, dejar vacío. |

Oprima **Guardar**. Si va a manejar más de una empresa, use el selector de
empresa en la parte superior para cambiar de una a otra; cada empresa tiene sus
propios trabajadores, periodos y recibos.

## 3. Alta de trabajador

En **Trabajadores** llene el formulario. Todas las fechas van en formato
`AAAA-MM-DD` (por ejemplo, `2026-01-15`) y los importes son pesos mexicanos
**antes de impuestos**. Si el salario es mensual, divídalo entre 30 para obtener
el salario diario.

| Etiqueta del campo | Ejemplo | Ayuda |
|--------------------|---------|-------|
| Nombre completo | `José Pérez Núñez` | Nombre y apellidos, sin abreviaturas. |
| CURP | `PENJ850101HDFRRN01` | 18 caracteres. Si es extranjero, use la CURP genérica `XEXX010101HNEXXXA4`. |
| RFC | `PENJ850101AB1` | 13 caracteres para persona física. |
| NSS (Número de Seguridad Social) | `12345678901` | 11 dígitos. Se obtiene del IMSS. |
| Número de empleado | `101` | Clave interna opcional de referencia; puede dejarla vacía. |
| Fecha de nacimiento | fecha (`AAAA-MM-DD`) | Fecha en que nació el trabajador. |
| Estado civil | `Soltero/a` | Lista desplegable: soltero, casado, unión libre, divorciado o viudo. |
| Fecha de ingreso a la empresa | fecha (`AAAA-MM-DD`) | Fecha en que empezó a laborar en **esta** empresa. No es la fecha de nacimiento ni la fecha actual. |
| Tipo de contrato | `Indefinido` | Modalidad de contratación; define indemnizaciones en finiquitos. |
| Departamento | `Producción` | Área interna donde labora. |
| Puesto | `Operadora` | Nombre del puesto o cargo. |
| Salario diario (pesos) | `500.00` | Salario pactado, antes de impuestos. Si es mensual, dividir entre 30. |
| Periodicidad de pago | `Mensual` | Frecuencia con la que se paga al trabajador. |
| Jornada laboral | `Diurna` | Turno en que trabaja; afecta el cálculo de horas extra. |
| Horas semanales | `48` | Horas trabajadas por semana. Jornada completa = 48. |
| INFONAVIT activo | casilla | Activar solo si tiene crédito INFONAVIT vigente. Al marcarla aparecen los tres campos siguientes. |
| Número de crédito INFONAVIT | `1234567890` | Número de contrato otorgado por el INFONAVIT. |
| Factor de descuento | `0.5` | Valor del descuento: porcentaje, monto en pesos o VSM según el tipo elegido. |
| Tipo de descuento | `Cuota fija en VSM` | Porcentaje del salario, monto fijo en pesos o cuota fija en salarios mínimos (VSM). |

Guarde con el botón **Guardar trabajador**. Una vez dada de alta, aparecerá en
la lista de trabajadores al calcular la nómina.

## 4. Captura del periodo y cálculo del recibo

En **Nomina**, cada campo lleva su etiqueta, ejemplo y ayuda debajo:

| Etiqueta del campo | Qué significa |
|--------------------|---------------|
| Tipo de periodo | Semanal = 7 días, quincenal = 15, decenal = 10 y mensual = 30. Al elegirlo se actualizan los días pagados. |
| Fecha de inicio del periodo | Primer día que se va a pagar. Si la quincena es del 1 al 15 de enero, es `2026-01-01`. Formato `AAAA-MM-DD`. |
| Fecha de fin del periodo | Último día que se va a pagar. En el ejemplo, `2026-01-15`. Formato `AAAA-MM-DD`. |
| Fecha de pago | Día en que se paga al trabajador; puede ser igual o posterior al fin del periodo. La aplicación usa las tablas fiscales vigentes en esa fecha (UMA, subsidio e ISR). Formato `AAAA-MM-DD`. |
| Días pagados (calculado) | Se calcula solo a partir del tipo de periodo. **No se edita** (quincenal = 15 días). |
| Horas extraordinarias (horas) | Total de horas extra del periodo; use 0 si no hay. |
| Semanas con horas extra | Número de semanas en que se repitieron esas horas. |
| Jornada de las horas extra | Diurna, nocturna o mixta; define el factor de pago. |
| Horas extra al salario mínimo (ISR 93 I) | Marque la casilla solo si aplica la exención del artículo 93 fracción I de la LISR. |
| Trabajadores a incluir | Marque la casilla de cada trabajador que debe aparecer en este periodo; puede seleccionar varios. |

Pasos:

1. Elija el **Tipo de periodo**.
2. Capture la **Fecha de inicio del periodo**, la **Fecha de fin del periodo** y
   la **Fecha de pago** (esta última determina la tabla fiscal vigente).
3. Confirme los **Días pagados (calculado)**; no se escriben a mano.
4. Si hubo tiempo extraordinario, capture las horas, las semanas y la jornada.
5. Oprima **Crear periodo**. Verá el número de periodo asignado.
6. Marque la casilla de cada trabajador en **Trabajadores a incluir**.
7. Oprima **Calcular nómina**. Aparecerá una tabla con percepciones, ISR, IMSS,
   INFONAVIT, neto y costo patronal por trabajador.

Revise siempre el neto contra su cálculo de SUA antes de pagar. El ISR usa la
tabla vigente a la fecha de pago; por eso un periodo de enero 2026 utiliza la
UMA y el subsidio de enero, aunque lo calcule en noviembre.

### PDF del recibo

En la tabla de resultados de **Nomina**, cada renglón tiene un botón
**Descargar PDF** en la última columna. Para exportar un recibo:

1. Calcule la nómina como se describió arriba.
2. En el renglón del trabajador, haga clic en **Descargar PDF**.
3. Se guardará un archivo `recibo_<id>.pdf` (por ejemplo,
   `recibo_7.pdf`) en su carpeta de descargas.
4. Ábralo en Preview (macOS) o Edge (Windows) y compruebe que el nombre
   `José Pérez Núñez` y la palabra `Gratificación`, si aparece, conserven los
   acentos.

El nombre del trabajador se toma del alta correspondiente; si no se muestra,
verifique que el campo **Nombre** esté lleno en **Trabajadores** y vuelva a
calcular la nómina.

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

Los campos y sus ayudas son:

| Etiqueta del campo | Ejemplo / ayuda |
|--------------------|-----------------|
| Tipo de cálculo | Finiquito (renuncia) o Liquidación (despido injustificado). |
| Fecha de baja | Último día laborado del trabajador. Formato `AAAA-MM-DD`. |
| Tipo de separación | Lista: renuncia, despido justificado, despido injustificado, terminación de contrato, muerte o incapacidad. |
| Salario diario (pesos) | `500.00`. Salario antes de impuestos. |
| Días de salario pendientes (días) | `0`. Días laborados aún no pagados. |
| Días de vacaciones pendientes (días) | `0`. Vacaciones no disfrutadas que se liquidan. |
| Días trabajados en el año (días) | `180`. Se usa para aguinaldo y prima vacacional. |
| Años de antigüedad (años) | `2`. Solo se usa en liquidaciones. |
| Motivo (opcional) | `Renuncia voluntaria`. Descripción libre de la baja. |

Capture los datos y oprima **Calcular**. La fecha de baja, el tipo de separación
y el motivo se muestran en el resultado para su expediente.

## 7. Recálculo retroactivo

Use **Recálculo** cuando haya cambiado un salario o una prestación después de
cerrar un periodo y necesite corregir los recibos.

- En **Periodo a recalcular** elija un periodo de la lista; se muestran su
  número, fecha de pago y estado.
- En el área de texto **Motivo del recálculo (obligatorio)** explique por qué se
  recalcula: cambio de salario, corrección de error, ajuste de UMA, etc. Queda
  registrado en el log de auditoría. No se permite dejarlo vacío. Ejemplo:
  `Corrección de salario posterior al cierre`.
- En **Trabajador individual (opcional)** puede elegir un solo trabajador para
  recalcular únicamente su recibo dentro del periodo.
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

## 7.1 Log de auditoría

En **Auditoría** puede filtrar el registro inmutable de cambios. Todos los
filtros tienen etiqueta, ejemplo y ayuda visible:

| Etiqueta del campo | Ejemplo / ayuda |
|--------------------|-----------------|
| Tabla | Lista: trabajadores, recibos, periodos_nomina, incapacidades o vacaciones. |
| Operación | Alta (INSERT), cambio (UPDATE), baja (DELETE) o recálculo (RECALCULO). |
| Usuario | `system`. Usuario de auditoría que hizo el cambio. |
| Fecha desde | Primer día del rango, en UTC. Formato `AAAA-MM-DD`. |
| Fecha hasta | Último día del rango, en UTC. Formato `AAAA-MM-DD`. |

Los registros se muestran de diez en diez con fecha UTC, tabla, operación,
registro, usuario y el resumen de los datos antes y después del cambio.

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

**¿Qué hago si en macOS dice "NominaMX se cerró inesperadamente"?** Instale el
bundle corregido. Si necesita abrir de inmediato el anterior, renombre la base
en Terminal:

```bash
mv "$HOME/Library/Application Support/com.nomina.mx/nomina.db" \
   "$HOME/Library/Application Support/com.nomina.mx/nomina.db.antiguo"
```

Luego abra NominaMX. El archivo antiguo queda con sufijo `.antiguo` y no se
borra ningún dato de negocio.

## 10. Registro visual de los formularios etiquetados

Las tablas de las secciones 2, 3, 4, 6, 7 y 7.1 son la descripción textual de
cada campo (etiqueta, ejemplo y ayuda). Las capturas de pantalla se tomarán
durante la **reinstalación del `.dmg` en Mac Intel y verificación visual**, y se
guardarán en `docs/evidencia/verificacion_visual/macos/` junto al resto de
evidencias de P0-C. Hasta entonces no se declara release ni se inicia el piloto.




