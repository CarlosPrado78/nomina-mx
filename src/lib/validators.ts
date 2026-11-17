export const rfcRegex = /^[A-ZÑ&]{3,4}\d{6}[A-Z0-9]{3}$/;
export const curpRegex = /^[A-Z][AEIOUX][A-Z]{2}\d{6}[HM][A-Z]{5}[A-Z0-9]\d$/;
export const nssRegex = /^\d{11}$/;

export function validarRFC(v: string) { return rfcRegex.test(v.toUpperCase()); }
export function validarCURP(v: string) { return curpRegex.test(v.toUpperCase()); }
export function validarNSS(v: string) { return nssRegex.test(v); }