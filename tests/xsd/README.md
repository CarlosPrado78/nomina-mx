# XSD del SAT — procedencia y localización

Fecha de descarga: 2026-11-16.

Los archivos se descargaron de los endpoints públicos del SAT:

- `cfdv40.xsd` — https://www.sat.gob.mx/sitio_internet/cfd/4/cfdv40.xsd
- `nomina12.xsd` — https://www.sat.gob.mx/sitio_internet/cfd/nomina/nomina12.xsd
- `catCFDI.xsd` — https://www.sat.gob.mx/sitio_internet/cfd/catalogos/catCFDI.xsd
- `tdCFDI.xsd` — https://www.sat.gob.mx/sitio_internet/cfd/tipoDatos/tdCFDI/tdCFDI.xsd
- `catNomina.xsd` — https://www.sat.gob.mx/sitio_internet/cfd/catalogos/Nomina/catNomina.xsd

Después de la descarga se reescribieron todos los `schemaLocation` de URLs HTTP/HTTPS a rutas relativas locales, para que `xmllint` nunca resuelva imports por red:

- `http://www.sat.gob.mx/sitio_internet/cfd/catalogos/catCFDI.xsd` → `catCFDI.xsd`
- `http://www.sat.gob.mx/sitio_internet/cfd/tipoDatos/tdCFDI/tdCFDI.xsd` → `tdCFDI.xsd`
- `http://www.sat.gob.mx/sitio_internet/cfd/catalogos/Nomina/catNomina.xsd` → `catNomina.xsd`

`all.xsd` es un esquema agregador que importa `cfdv40.xsd` y `nomina12.xsd` por ruta relativa.

Verificación:

```bash
grep -rn "schemaLocation" tests/xsd/
# Ninguno apunta a http:// ni https://

xmllint --noout --schema tests/xsd/all.xsd /tmp/cfdi_test.xml
# /tmp/cfdi_test.xml validates
```
