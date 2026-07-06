import { useEffect } from "react";
import { HashRouter, Routes, Route, NavLink, useLocation } from "react-router-dom";
import { Home, Activity, Database, Eye, Settings as SettingsIcon } from "lucide-react";
import HomePage from "./pages/HomePage";
import SeriesDetailPage from "./pages/SeriesDetailPage";
import SettingsTab from "./components/SettingsTab";
import MetadataTab from "./pages/MetadataTab";
import VideoPlayer from "./pages/VideoPlayer";
import FeedPage from "./pages/FeedPage";
import WatchingPage from "./pages/WatchingPage";
import SubscriptionsPage from "./pages/SubscriptionsPage";
import ContextMenu from "./components/ContextMenu";
import { ActivityLogProvider } from "./contexts/ActivityLogContext";
import { ActivityLogDrawer } from "./components/ActivityLogDrawer";
import { TitleLanguageProvider } from "./contexts/TitleLanguageContext";
import { HiddenShowsProvider } from "./contexts/HiddenShowsContext";
import { TrackerProgressProvider } from "./contexts/TrackerProgressContext";
import { ViewHistoryProvider } from "./contexts/ViewHistoryContext";
import LangSwitch from "./components/LangSwitch";
import AmbientCursor from "./components/AmbientCursor";
import appIcon from "../../assets/icon.png";
import pkg from "../../package.json";
import "./styles/App.css";

const VERSION = pkg.version;

// Tiny route → window-title map. Series and player paths get a generic
// label here; deeper pages can override their own title with a useEffect
// in the page component if they want a per-series taskbar entry.
function titleForPath(pathname: string): string {
  if (pathname === "/") return "AniBeam - Library";
  if (pathname.startsWith("/series/")) return "AniBeam - Series";
  if (pathname.startsWith("/feed")) return "AniBeam - Feed";
  if (pathname.startsWith("/watching")) return "AniBeam - Watching";
  if (pathname.startsWith("/subscriptions")) return "AniBeam - Subscriptions";
  if (pathname.startsWith("/metadata")) return "AniBeam - Metadata";
  if (pathname.startsWith("/settings")) return "AniBeam - Settings";
  if (pathname.startsWith("/player/")) return "AniBeam - Player";
  return "AniBeam";
}

function AppContent() {
  const location = useLocation();
  const isPlayerRoute = location.pathname.startsWith("/player/");
  const isLib = location.pathname === "/" || location.pathname.startsWith("/series/");

  useEffect(() => {
    document.title = titleForPath(location.pathname);
  }, [location.pathname]);

  return (
    <div className="app">
      {!isPlayerRoute && <AmbientCursor />}
      {!isPlayerRoute && (
        <aside className="rail">
          <NavLink to="/" end className="rail-brand" data-halo-snap aria-label="Go to Library">
            <img src={appIcon} alt="" draggable={false} />
          </NavLink>
          <nav className="rail-nav">
            <NavLink to="/" end className={`rail-link${isLib ? " active" : ""}`} data-halo-snap>
              <Home size={18} />
              <span className="rail-link-label">Library</span>
            </NavLink>
            <NavLink to="/feed" className={({ isActive }) => `rail-link${isActive ? " active" : ""}`} data-halo-snap>
              <Activity size={18} />
              <span className="rail-link-label">Feed</span>
            </NavLink>
            <NavLink to="/watching" className={({ isActive }) => `rail-link${isActive ? " active" : ""}`} data-halo-snap>
              <Eye size={18} />
              <span className="rail-link-label">Watching</span>
            </NavLink>
            <NavLink to="/metadata" className={({ isActive }) => `rail-link${isActive ? " active" : ""}`} data-halo-snap>
              <Database size={18} />
              <span className="rail-link-label">Metadata</span>
            </NavLink>
            <NavLink to="/settings" className={({ isActive }) => `rail-link${isActive ? " active" : ""}`} data-halo-snap>
              <SettingsIcon size={18} />
              <span className="rail-link-label">Settings</span>
            </NavLink>
          </nav>
          <div className="rail-foot">
            <LangSwitch />
            <span className="rail-meta">v{VERSION}</span>
          </div>
        </aside>
      )}
      {!isPlayerRoute ? (
        <main className="main-content">
          <Routes>
            <Route path="/" element={<HomePage />} />
            <Route path="/feed" element={<FeedPage />} />
            <Route path="/watching" element={<WatchingPage />} />
            <Route path="/subscriptions" element={<SubscriptionsPage />} />
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
      <TitleLanguageProvider>
        <HiddenShowsProvider>
          <TrackerProgressProvider>
            <ViewHistoryProvider>
              <ActivityLogProvider>
                <AppContent />
              </ActivityLogProvider>
            </ViewHistoryProvider>
          </TrackerProgressProvider>
        </HiddenShowsProvider>
      </TitleLanguageProvider>
    </HashRouter>
  );
}

export default App;
