import { useEffect } from 'react';
import { Routes, Route, NavLink } from 'react-router-dom';
import EmpresaPage from './pages/Empresa';
import TrabajadoresPage from './pages/Trabajadores';
import NominaPage from './pages/Nomina';
import FiniquitosPage from './pages/Finiquitos';
import ReportesPage from './pages/Reportes';
import ConfigPage from './pages/Configuracion';
import AuditoriaPage from './pages/Auditoria';
import EmpresaSelector from './pages/EmpresaSelector';
import ImportarPage from './pages/Importar';
import RecalculoPage from './pages/Recalculo';
import { api } from './lib/tauri';

export default function App() {
  // Fija el usuario de auditoría (NOMINA_USUARIO / config.json / "system")
  // antes de que el usuario interactúe con el resto de pantallas.
  useEffect(() => {
    api.configurarUsuarioAuditoria().catch(console.error);
  }, []);

  return (
    <div className="min-h-screen bg-gray-50">
      <nav className="bg-white shadow p-4 flex flex-wrap items-center gap-4">
        <NavLink to="/empresa">Empresa</NavLink>
        <NavLink to="/trabajadores">Trabajadores</NavLink>
        <NavLink to="/importar">Importar CSV</NavLink>
        <NavLink to="/nomina">Nomina</NavLink>
        <NavLink to="/recalculo">Recálculo</NavLink>
        <NavLink to="/finiquitos">Finiquitos</NavLink>
        <NavLink to="/reportes">Reportes</NavLink>
        <NavLink to="/config">Configuracion</NavLink>
        <NavLink to="/auditoria">Auditoría</NavLink>
        <EmpresaSelector />
        <div className="ml-auto flex flex-col items-end leading-tight">
          <NavLink to="/" className="font-bold">NominaMX</NavLink>
          <p className="text-xs text-muted-foreground">
            Developed by Carlos G. Prado Cabrera
          </p>
        </div>
      </nav>
      <main className="p-6">
        <Routes>
          <Route path="/" element={<EmpresaPage />} />
          <Route path="/empresa" element={<EmpresaPage />} />
          <Route path="/trabajadores" element={<TrabajadoresPage />} />
          <Route path="/importar" element={<ImportarPage />} />
          <Route path="/nomina" element={<NominaPage />} />
          <Route path="/recalculo" element={<RecalculoPage />} />
          <Route path="/finiquitos" element={<FiniquitosPage />} />
          <Route path="/reportes" element={<ReportesPage />} />
          <Route path="/config" element={<ConfigPage />} />
          <Route path="/auditoria" element={<AuditoriaPage />} />
        </Routes>
      </main>
    </div>
  );
}