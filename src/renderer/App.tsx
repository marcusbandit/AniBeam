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
import { useMpvPlaybackSync } from "./hooks/useMpvPlaybackSync";
import { useScrollRestore } from "./hooks/useScrollRestore";
import { MAIN_SCROLL_ID, RAIL_TABS, activeTab, readTrail } from "./hooks/navTrailCore";
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

// Icon per rail destination. The tab list itself lives in navTrailCore, which
// stays React-free so the verify script can import it; icons are components,
// so the pairing is completed here. Keys are RAIL_TABS paths.
const RAIL_ICONS: Record<string, typeof Home> = {
  "/": Home,
  "/feed": Activity,
  "/watching": Eye,
  "/metadata": Database,
  "/settings": SettingsIcon,
};

function AppContent() {
  const location = useLocation();
  const isPlayerRoute = location.pathname.startsWith("/player/");
  // One highlight decision for the whole rail, off the trail rather than off
  // NavLink's own isActive: a series page has no rail route of its own and
  // belongs to whichever tab the user browsed in from. Letting both mechanisms
  // run would light two links at once.
  const tab = activeTab(location.pathname, readTrail(location.state));

  // App-wide, not per-page: an mpv window routinely outlives the page that
  // launched it, and its resume position still has to land.
  useMpvPlaybackSync();
  // Same reasoning: `.main-content` is one element shared by every route, so
  // its scroll position has to be owned above the pages, not inside them.
  useScrollRestore();

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
            {RAIL_TABS.map((t) => {
              const Icon = RAIL_ICONS[t.path];
              return (
                <NavLink
                  key={t.path}
                  to={t.path}
                  end={t.path === "/"}
                  className={`rail-link${tab === t.path ? " active" : ""}`}
                  data-halo-snap
                >
                  <Icon size={18} />
                  <span className="rail-link-label">{t.label}</span>
                </NavLink>
              );
            })}
          </nav>
          <div className="rail-foot">
            <LangSwitch />
            <span className="rail-meta">v{VERSION}</span>
          </div>
        </aside>
      )}
      {!isPlayerRoute ? (
        <main id={MAIN_SCROLL_ID} className="main-content">
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
