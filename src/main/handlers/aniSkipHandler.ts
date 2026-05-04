import { logger } from '../services/logger';
import metadataHandler from './metadataHandler';

export interface SkipTime {
  start: number;
  end: number;
}

export interface SkipTimes {
  op?: SkipTime;
  ed?: SkipTime;
}

interface AniSkipResponse {
  found?: boolean;
  results?: Array<{
    interval: { startTime: number; endTime: number };
    skipType: 'op' | 'ed' | 'mixed-op' | 'mixed-ed' | 'recap';
  }>;
  message?: string;
}

const aniSkipHandler = {
  /**
   * Fetch intro/outro skip times for one episode from the AniSkip community
   * database, then write them onto the matching entry in metadata.json. Marks
   * the episode as `skipFetched` so we don't refetch on every play (a miss is
   * a stable answer for that episode/length combination).
   */
  async fetchAndCache(
    seriesId: string,
    malId: number,
    episodeNumber: number,
    episodeLength: number,
  ): Promise<SkipTimes> {
    if (!malId || !episodeNumber || !episodeLength || episodeLength <= 0) {
      return {};
    }
    const url = `https://api.aniskip.com/v2/skip-times/${malId}/${episodeNumber}?types[]=op&types[]=ed&episodeLength=${Math.round(episodeLength)}`;
    let times: SkipTimes = {};
    try {
      const resp = await fetch(url);
      if (!resp.ok) {
        if (resp.status !== 404) {
          logger.warn('metadata', `AniSkip ${resp.status} for malId=${malId} ep=${episodeNumber}`);
        }
      } else {
        const data = (await resp.json()) as AniSkipResponse;
        for (const r of data.results ?? []) {
          if (r.skipType === 'op' || r.skipType === 'mixed-op') {
            times.op = { start: r.interval.startTime, end: r.interval.endTime };
          } else if (r.skipType === 'ed' || r.skipType === 'mixed-ed') {
            times.ed = { start: r.interval.startTime, end: r.interval.endTime };
          }
        }
        const summary = [
          times.op ? `op=[${times.op.start.toFixed(0)},${times.op.end.toFixed(0)}]` : null,
          times.ed ? `ed=[${times.ed.start.toFixed(0)},${times.ed.end.toFixed(0)}]` : null,
        ].filter(Boolean).join(' ');
        logger.info('metadata', `AniSkip ${data.found ? 'hit' : 'miss'} mal=${malId} ep=${episodeNumber}${summary ? ' ' + summary : ''}`);
      }
    } catch (err) {
      logger.warn('metadata', `AniSkip request failed: ${(err as Error).message}`);
      // Don't mark as fetched on network error — let it retry next play.
      return times;
    }

    // Persist on the episode entry inside metadata.json.
    try {
      await metadataHandler.transaction(async (meta) => {
        const series = meta[seriesId] as
          | { episodes?: Array<{ episodeNumber: number; opStart?: number; opEnd?: number; edStart?: number; edEnd?: number; skipFetched?: boolean }> }
          | undefined;
        if (!series?.episodes) return { result: undefined, updated: null };
        const ep = series.episodes.find((e) => e.episodeNumber === episodeNumber);
        if (!ep) return { result: undefined, updated: null };
        ep.opStart = times.op?.start;
        ep.opEnd = times.op?.end;
        ep.edStart = times.ed?.start;
        ep.edEnd = times.ed?.end;
        ep.skipFetched = true;
        return { result: undefined, updated: meta };
      });
    } catch (err) {
      logger.warn('metadata', `AniSkip cache write failed: ${(err as Error).message}`);
    }
    return times;
  },
};

export default aniSkipHandler;
