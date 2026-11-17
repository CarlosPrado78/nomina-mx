# Firma y distribucion

## macOS (notarytool)
1. Crear Apple Developer ID Application certificate.
2. Configurar variables:
   - APPLE_ID, APPLE_TEAM_ID, APPLE_APP_SPECIFIC_PASSWORD
3. Build:
   ```bash
   pnpm tauri:build
   ```
4. Notarizar:
   ```bash
   xcrun notarytool submit src-tauri/target/release/bundle/dmg/NominaMX_*.dmg \
     --apple-id "$APPLE_ID" --team-id "$APPLE_TEAM_ID" \
     --password "$APPLE_APP_SPECIFIC_PASSWORD" --wait
   ```
5. Staple:
   ```bash
   xcrun stapler staple src-tauri/target/release/bundle/dmg/NominaMX_*.dmg
   ```

## Windows (signtool)
1. Obtener certificado de firma de codigo (PFX).
2. Build MSI:
   ```bash
   pnpm tauri:build
   ```
3. Firmar:
   ```bash
   signtool sign /f cert.pfx /p <password> /tr http://timestamp.digicert.com /td sha256 /fd sha256 src-tauri\target\release\bundle\msi\NominaMX_*.msi
   ```
4. Verificar:
   ```bash
   signtool verify /pa src-tauri\target\release\bundle\msi\NominaMX_*.msi
   ```

## Estado Fase 8
Estructura lista, firma pendiente de certificados. (PARCIAL hasta contar con certificados).