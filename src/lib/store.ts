import { create } from 'zustand';

interface AppState {
  empresaId: number | null;
  setEmpresaId: (id: number | null) => void;
}

export const useAppStore = create<AppState>((set) => ({
  empresaId: null,
  setEmpresaId: (id) => set({ empresaId: id }),
}));