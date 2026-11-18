import { create } from 'zustand';

interface AppState {
  empresaId: number | null;
  empresaActiva: number | null;
  /** Se incrementa al cambiar de empresa para que las pantallas recarguen. */
  reloadKey: number;
  setEmpresaId: (id: number | null) => void;
  setEmpresaActiva: (id: number | null) => void;
  recargar: () => void;
}

export const useAppStore = create<AppState>((set) => ({
  empresaId: null,
  empresaActiva: null,
  reloadKey: 0,
  setEmpresaId: (id) => set({ empresaId: id }),
  setEmpresaActiva: (id) => set({ empresaActiva: id, empresaId: id }),
  recargar: () => set((s) => ({ reloadKey: s.reloadKey + 1 })),
}));
