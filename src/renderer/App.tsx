import { HashRouter, Routes, Route, NavLink, useLocation } from "react-router-dom";
import { Tv, Home, Activity, Database, Settings as SettingsIcon } from "lucide-react";
import HomePage from "./pages/HomePage";
import SeriesDetailPage from "./pages/SeriesDetailPage";
import SettingsTab from "./components/SettingsTab";
import MetadataTab from "./pages/MetadataTab";
import VideoPlayer from "./pages/VideoPlayer";
import FeedPage from "./pages/FeedPage";
import ContextMenu from "./components/ContextMenu";
import { ActivityLogProvider } from "./contexts/ActivityLogContext";
import { ActivityLogDrawer } from "./components/ActivityLogDrawer";
import "./styles/App.css";

function AppContent() {
  const location = useLocation();
  const isPlayerRoute = location.pathname.startsWith("/player/");
  const isLib = location.pathname === "/" || location.pathname.startsWith("/series/");

  return (
    <div className="app">
      {!isPlayerRoute && (
        <nav className="navbar">
          <div className="navbar-brand">
            <span className="brand-mark"><Tv size={16} strokeWidth={2.25} /></span>
            <span className="brand-word">AniBeam</span>
          </div>
          <div className="navbar-nav">
            <NavLink to="/" end className={`nav-link${isLib ? " active" : ""}`}>
              <Home size={15} />
              <span>Library</span>
            </NavLink>
            <NavLink to="/feed" className={({ isActive }) => `nav-link${isActive ? " active" : ""}`}>
              <Activity size={15} />
              <span>Feed</span>
            </NavLink>
            <NavLink to="/metadata" className={({ isActive }) => `nav-link${isActive ? " active" : ""}`}>
              <Database size={15} />
              <span>Metadata</span>
            </NavLink>
            <NavLink to="/settings" className={({ isActive }) => `nav-link${isActive ? " active" : ""}`}>
              <SettingsIcon size={15} />
              <span>Settings</span>
            </NavLink>
          </div>
          <span className="navbar-meta">v0.0.0</span>
        </nav>
      )}
      {!isPlayerRoute ? (
        <main className="main-content">
          <Routes>
            <Route path="/" element={<HomePage />} />
            <Route path="/feed" element={<FeedPage />} />
            <Route path="/series/:seriesId" element={<SeriesDetailPage />} />
            <Route path="/settings" element={<SettingsTab />} />
            <Route path="/metadata" element={<MetadataTab />} />
          </Routes>
        </main>
      ) : (
        <Routes>
          <Route path="/player/:seriesId/:episodeNumber" element={<VideoPlayer />} />
        </Routes>
      )}
      {!isPlayerRoute && <ContextMenu />}
      <ActivityLogMount />
    </div>
  );
}

function ActivityLogMount() {
  const { pathname } = useLocation();
  const visible = pathname === "/settings" || pathname === "/metadata";
  if (!visible) return null;
  return <ActivityLogDrawer />;
}

function App() {
  return (
    <HashRouter>
      <ActivityLogProvider>
        <AppContent />
      </ActivityLogProvider>
    </HashRouter>
  );
}

export default App;
