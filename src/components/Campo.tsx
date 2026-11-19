import { cloneElement, isValidElement, type ReactElement, type ReactNode } from 'react';

interface CampoProps {
  /** Identificador único del control; se inyecta en el hijo. */
  id: string;
  /** Etiqueta visible, en español, con el nombre exacto del dato. */
  label: string;
  /** Ayuda de una línea debajo del campo (qué se espera, formato o unidades). */
  hint: string;
  /** Un único input, select o textarea. Recibe id y aria-describedby. */
  children: ReactNode;
  className?: string;
}

/**
 * Envuelve un control con <label> explícito y texto de ayuda asociado.
 * El id y aria-describedby se inyectan al hijo para que no se puedan olvidar.
 */
export function Campo({ id, label, hint, children, className = '' }: CampoProps) {
  const control = isValidElement(children)
    ? cloneElement(children as ReactElement<any>, {
        id,
        'aria-describedby': `${id}-hint`,
      })
    : children;

  return (
    <div className={`flex flex-col gap-1 ${className}`}>
      <label htmlFor={id} className="text-sm font-semibold text-gray-700">
        {label}
      </label>
      {control}
      <p id={`${id}-hint`} className="text-xs text-gray-500">
        {hint}
      </p>
    </div>
  );
}

export default Campo;
