import { useEffect } from 'react';
import { Routes, Route, NavLink } from 'react-router-dom';
import EmpresaPage from './pages/Empresa';
import TrabajadoresPage from './pages/Trabajadores';
import NominaPage from './pages/Nomina';
import FiniquitosPage from './pages/Finiquitos';
import ReportesPage from './pages/Reportes';
import ConfigPage from './pages/Configuracion';
import AuditoriaPage from './pages/Auditoria';
import { api } from './lib/tauri';

export default function App() {
  // Fija el usuario de auditoría (NOMINA_USUARIO / config.json / "system")
  // antes de que el usuario interactúe con el resto de pantallas.
  useEffect(() => {
    api.configurarUsuarioAuditoria().catch(console.error);
  }, []);

  return (
    <div className="min-h-screen bg-gray-50">
      <nav className="bg-white shadow p-4 flex gap-4">
        <NavLink to="/" className="font-bold">NominaMX</NavLink>
        <NavLink to="/empresa">Empresa</NavLink>
        <NavLink to="/trabajadores">Trabajadores</NavLink>
        <NavLink to="/nomina">Nomina</NavLink>
        <NavLink to="/finiquitos">Finiquitos</NavLink>
        <NavLink to="/reportes">Reportes</NavLink>
        <NavLink to="/config">Configuracion</NavLink>
        <NavLink to="/auditoria">Auditoría</NavLink>
      </nav>
      <main className="p-6">
        <Routes>
          <Route path="/" element={<EmpresaPage />} />
          <Route path="/empresa" element={<EmpresaPage />} />
          <Route path="/trabajadores" element={<TrabajadoresPage />} />
          <Route path="/nomina" element={<NominaPage />} />
          <Route path="/finiquitos" element={<FiniquitosPage />} />
          <Route path="/reportes" element={<ReportesPage />} />
          <Route path="/config" element={<ConfigPage />} />
          <Route path="/auditoria" element={<AuditoriaPage />} />
        </Routes>
      </main>
    </div>
  );
}