"use strict";
const electron = require("electron");
electron.contextBridge.exposeInMainWorld("electronAPI", {
  // Config
  getFolderSources: () => electron.ipcRenderer.invoke("get-folder-sources"),
  addFolderSource: (folderPath) => electron.ipcRenderer.invoke("add-folder-source", folderPath),
  removeFolderSource: (folderPath) => electron.ipcRenderer.invoke("remove-folder-source", folderPath),
  // Folder scanning
  selectFolder: () => electron.ipcRenderer.invoke("select-folder"),
  scanFolder: (folderPath) => electron.ipcRenderer.invoke("scan-folder", folderPath),
  scanAllFolders: () => electron.ipcRenderer.invoke("scan-all-folders"),
  scanAndFetchMetadata: (folderPath) => electron.ipcRenderer.invoke("scan-and-fetch-metadata", folderPath),
  libraryWalk: () => electron.ipcRenderer.invoke("library:walk"),
  // Metadata
  fetchMetadata: (seriesName) => electron.ipcRenderer.invoke("fetch-metadata", seriesName),
  fetchAnilistMetadata: (seriesName) => electron.ipcRenderer.invoke("fetch-anilist-metadata", seriesName),
  fetchMALMetadata: (seriesName) => electron.ipcRenderer.invoke("fetch-mal-metadata", seriesName),
  saveMetadata: (metadata) => electron.ipcRenderer.invoke("save-metadata", metadata),
  loadMetadata: () => electron.ipcRenderer.invoke("load-metadata"),
  clearMetadata: () => electron.ipcRenderer.invoke("clear-metadata"),
  deleteSeries: (seriesId) => electron.ipcRenderer.invoke("delete-series", seriesId),
  getSeriesEpisodes: (seriesId) => electron.ipcRenderer.invoke("get-series-episodes", seriesId),
  // Match picker
  searchAnilist: (query, limit) => electron.ipcRenderer.invoke("anilist:search", query, limit),
  applyAnilistMatch: (seriesId, anilistId, seasonNumber) => electron.ipcRenderer.invoke("metadata:apply-anilist-match", seriesId, anilistId, seasonNumber ?? null),
  // Image cache
  getImageCacheStats: () => electron.ipcRenderer.invoke("get-image-cache-stats"),
  clearImageCache: () => electron.ipcRenderer.invoke("clear-image-cache"),
  getImageCachePath: () => electron.ipcRenderer.invoke("get-image-cache-path"),
  // Activity log
  onLogEvent: (handler) => {
    const listener = (_e, event) => handler(event);
    electron.ipcRenderer.on("log:event", listener);
    return () => electron.ipcRenderer.removeListener("log:event", listener);
  },
  getLogBuffer: () => electron.ipcRenderer.invoke("log:get-buffer"),
  clearLog: () => electron.ipcRenderer.invoke("log:clear"),
  // Video probe
  probeRetry: (filePath) => electron.ipcRenderer.invoke("probe:retry", filePath),
  onMetadataFileStatusChanged: (handler) => {
    const listener = (_e, payload) => handler(payload);
    electron.ipcRenderer.on("metadata:file-status-changed", listener);
    return () => electron.ipcRenderer.removeListener("metadata:file-status-changed", listener);
  },
  // Embedded subtitles
  listEmbeddedSubtitles: (videoPath) => electron.ipcRenderer.invoke("subtitle:list-embedded", videoPath),
  extractEmbeddedSubtitle: (videoPath, streamIndex, codec) => electron.ipcRenderer.invoke("subtitle:extract", videoPath, streamIndex, codec),
  // AniSkip
  fetchSkipTimes: (seriesId, malId, episodeNumber, episodeLength) => electron.ipcRenderer.invoke("aniskip:fetch", seriesId, malId, episodeNumber, episodeLength),
  // Shell
  openExternal: (url) => electron.ipcRenderer.invoke("shell:open-external", url),
  openWithMpv: (filePath) => electron.ipcRenderer.invoke("shell:open-with-mpv", filePath),
  // Trackers
  trackerStatus: (provider) => electron.ipcRenderer.invoke("tracker:status", provider),
  trackerSetClientId: (provider, clientId) => electron.ipcRenderer.invoke("tracker:set-client-id", provider, clientId),
  trackerGetClientId: (provider) => electron.ipcRenderer.invoke("tracker:get-client-id", provider),
  trackerConnect: (provider, clientId, clientSecret) => electron.ipcRenderer.invoke("tracker:connect", provider, clientId, clientSecret ?? ""),
  trackerCancelConnect: () => electron.ipcRenderer.invoke("tracker:cancel-connect"),
  trackerDisconnect: (provider) => electron.ipcRenderer.invoke("tracker:disconnect", provider),
  trackerMarkEpisode: (provider, mediaId, episodeNumber, totalEpisodes) => electron.ipcRenderer.invoke("tracker:mark-episode", provider, mediaId, episodeNumber, totalEpisodes)
});
