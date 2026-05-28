import { useEffect, useMemo, useRef, useState } from 'react';
import type { SeriesMetadata } from './useMetadata';
import {
  closeGraph,
  type FranchiseGraph,
  type FranchiseNode,
  type RawRelation,
} from '../../shared/franchise';

/** Build ownedNodes + seedRelations from the in-memory owned-metadata map. */
function buildLocalSeed(allMeta: Record<string, SeriesMetadata>): {
  ownedNodes: Map<number, FranchiseNode>;
  seedRelations: Map<number, RawRelation[]>;
} {
  const ownedNodes = new Map<number, FranchiseNode>();
  const seedRelations = new Map<number, RawRelation[]>();
  for (const s of Object.values(allMeta)) {
    if (typeof s.anilistId !== 'number') continue;
    ownedNodes.set(s.anilistId, {
      anilistId: s.anilistId,
      malId: s.malId ?? null,
      type: 'ANIME',
      format: s.format ?? null,
      status: s.status ?? null,
      seasonYear: s.seasonYear ?? null,
      siteUrl: null,
      titleRomaji: s.titleRomaji ?? null,
      titleEnglish: s.titleEnglish ?? null,
      poster: s.poster ?? null,
    });
    if (Array.isArray(s.relations)) {
      seedRelations.set(s.anilistId, s.relations as unknown as RawRelation[]);
    }
  }
  return { ownedNodes, seedRelations };
}

export interface UseFranchiseGraph {
  graph: FranchiseGraph | null;
  /** True while the background AniList fill is in flight. */
  filling: boolean;
}

/**
 * Hybrid franchise-graph consumer. Synchronously builds a local graph from the
 * owned-metadata map for instant render, then requests the closed+cached graph
 * from the main process and swaps it in when it arrives.
 */
export function useFranchiseGraph(
  currentAnilistId: number | null | undefined,
  allMeta: Record<string, SeriesMetadata>,
): UseFranchiseGraph {
  const [filled, setFilled] = useState<FranchiseGraph | null>(null);
  const [filling, setFilling] = useState(false);
  const [localGraph, setLocalGraph] = useState<FranchiseGraph | null>(null);
  const reqIdRef = useRef(0);

  // Local seed — recomputed when the current series or metadata changes.
  useEffect(() => {
    let cancelled = false;
    if (currentAnilistId == null) { setLocalGraph(null); return; }
    const { ownedNodes, seedRelations } = buildLocalSeed(allMeta);
    const currentNode = ownedNodes.get(currentAnilistId) ?? {
      anilistId: currentAnilistId, malId: null, type: 'ANIME', format: null,
      status: null, seasonYear: null, siteUrl: null, titleRomaji: null,
      titleEnglish: null, poster: null,
    };
    void closeGraph({ seedNodes: [currentNode], seedRelations }).then((g) => {
      if (!cancelled) setLocalGraph(g);
    });
    return () => { cancelled = true; };
  }, [currentAnilistId, allMeta]);

  // Background fill from AniList (cached + progressively completed in main).
  useEffect(() => {
    setFilled(null);
    // Also clear the prior franchise's local seed so we never show the old
    // graph against the new current id for a frame (this effect keys on
    // currentAnilistId only, so metadata-ping re-renders don't flicker).
    setLocalGraph(null);
    if (currentAnilistId == null) return;
    const myReq = ++reqIdRef.current;
    setFilling(true);

    let timer: ReturnType<typeof setTimeout> | null = null;
    let stopped = false;
    const POLL_MS = 5000;

    const run = () => {
      void window.electronAPI
        .getFranchiseGraph(currentAnilistId)
        .then((g) => {
          if (stopped || reqIdRef.current !== myReq) return;
          setFilled(g);
          if (g && g.complete === false) {
            timer = setTimeout(run, POLL_MS);
          } else {
            setFilling(false);
          }
        })
        .catch(() => { if (!stopped && reqIdRef.current === myReq) setFilling(false); });
    };
    run();

    return () => {
      stopped = true;
      if (timer) clearTimeout(timer);
      if (reqIdRef.current === myReq) setFilling(false);
    };
  }, [currentAnilistId]);

  // Prefer the filled graph once present; fall back to the local seed.
  const graph = useMemo(() => filled ?? localGraph, [filled, localGraph]);
  return { graph, filling };
}
