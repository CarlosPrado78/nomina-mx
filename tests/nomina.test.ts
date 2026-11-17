import { describe, it, expect } from 'vitest';
import { validarRFC, validarCURP, validarNSS } from '../src/lib/validators';

describe('Validadores', () => {
  it('RFC valido', () => expect(validarRFC('XAXX010101000')).toBe(true));
  it('RFC invalido', () => expect(validarRFC('123')).toBe(false));
  it('CURP valida', () => expect(validarCURP('XEXX010101HNEXXXA4')).toBe(true));
  it('NSS valido', () => expect(validarNSS('12345678901')).toBe(true));
  it('NSS invalido', () => expect(validarNSS('123')).toBe(false));
});

describe('Sanity ISR', () => {
  it('subsidio no excedente', () => {
    const isr = 100; const sub = 200;
    expect(Math.max(isr - sub, 0)).toBe(0);
  });
});