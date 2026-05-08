import { useState, useEffect, useCallback, ReactNode } from "react";
import { useNavigate, useLocation } from "react-router-dom";
import { ChevronLeft, RefreshCw, FileText, ExternalLink } from "lucide-react";
import { useMetadata } from "../hooks/useMetadata";

interface MenuPosition {
  x: number;
  y: number;
}

interface ContextButtonProps {
  onClick: () => void;
  children: ReactNode;
}

function ContextButton({ onClick, children }: ContextButtonProps) {
  return (
    <button className="context-menu-item" onClick={onClick}>
      {children}
    </button>
  );
}

function ContextMenu() {
  const [visible, setVisible] = useState(false);
  const [position, setPosition] = useState<MenuPosition>({ x: 0, y: 0 });
  const [episodeFile, setEpisodeFile] = useState<string | null>(null);
  const navigate = useNavigate();
  const location = useLocation();
  const { metadata, loadMetadata } = useMetadata();

  // Extract seriesId from pathname (e.g., /series/12345 -> 12345)
  const seriesIdMatch = location.pathname.match(/^\/series\/([^/]+)/);
  const seriesId = seriesIdMatch ? seriesIdMatch[1] : null;
  const isSeriesDetailPage = !!seriesId;

  const handleContextMenu = useCallback((e: MouseEvent) => {
    e.preventDefault();

    // If the right-click landed on an episode card with a known file, surface
    // file-specific actions (mpv launch). The data attribute is the contract.
    const targetEl = e.target instanceof Element ? e.target : null;
    const fileFromEpisode = targetEl?.closest('[data-episode-file]')?.getAttribute('data-episode-file') ?? null;
    setEpisodeFile(fileFromEpisode);

    // Position the menu at the cursor, but keep it within viewport
    const x = Math.min(e.clientX, window.innerWidth - 188);
    const y = Math.min(e.clientY, window.innerHeight - 120);

    setPosition({ x, y });
    setVisible(true);
  }, []);

  const handleClick = useCallback(() => {
    setVisible(false);
  }, []);

  const handleKeyDown = useCallback((e: KeyboardEvent) => {
    if (e.key === "Escape") {
      setVisible(false);
    }
  }, []);

  const handleBack = useCallback(() => {
    setVisible(false);
    navigate(-1);
  }, [navigate]);

  const handleRescanShow = useCallback(async () => {
    if (!seriesId || !isSeriesDetailPage) return;

    setVisible(false);

    const seriesData = metadata[seriesId];
    if (!seriesData?.folderPath) {
      alert("Cannot rescan: folder path not found for this series.");
      return;
    }

    try {
      await window.electronAPI.scanAndFetchMetadata(seriesData.folderPath);
      await loadMetadata();
      alert("Show rescanned successfully!");
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : "Unknown error";
      console.error("Error rescanning show:", err);
      alert(`Error rescanning show: ${errorMessage}`);
    }
  }, [seriesId, isSeriesDetailPage, metadata, loadMetadata]);

  const handleToMetadata = useCallback(() => {
    setVisible(false);
    navigate("/metadata");
  }, [navigate]);

  const handleOpenWithMpv = useCallback(async () => {
    if (!episodeFile) return;
    setVisible(false);
    try {
      await window.electronAPI.openWithMpv(episodeFile);
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : "Unknown error";
      console.error("Error launching mpv:", err);
      alert(`Could not launch mpv: ${errorMessage}`);
    }
  }, [episodeFile]);

  useEffect(() => {
    document.addEventListener("contextmenu", handleContextMenu);
    document.addEventListener("click", handleClick);
    document.addEventListener("keydown", handleKeyDown);

    return () => {
      document.removeEventListener("contextmenu", handleContextMenu);
      document.removeEventListener("click", handleClick);
      document.removeEventListener("keydown", handleKeyDown);
    };
  }, [handleContextMenu, handleClick, handleKeyDown]);

  if (!visible) return null;

  return (
    <div
      className="context-menu"
      style={{
        left: position.x,
        top: position.y,
      }}
    >
      <ContextButton onClick={handleBack}>
        <ChevronLeft className="context-menu-icon" size={18} />
        <span>Back</span>
      </ContextButton>
      {episodeFile && (
        <>
          <div className="context-menu-divider" />
          <ContextButton onClick={handleOpenWithMpv}>
            <ExternalLink className="context-menu-icon" size={18} />
            <span>Open with mpv</span>
          </ContextButton>
        </>
      )}
      {isSeriesDetailPage && (
        <>
          <div className="context-menu-divider" />
          <ContextButton onClick={handleRescanShow}>
            <RefreshCw className="context-menu-icon" size={18} />
            <span>Rescan Show</span>
          </ContextButton>
          <ContextButton onClick={handleToMetadata}>
            <FileText className="context-menu-icon" size={18} />
            <span>To Metadata</span>
          </ContextButton>
        </>
      )}
    </div>
  );
}

export default ContextMenu;
