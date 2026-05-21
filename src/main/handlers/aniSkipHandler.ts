import { logger } from '../services/logger';
import metadataHandler from './metadataHandler';
import { probeChapters, classifyChapters } from '../utils/chapterProbe';

export interface SkipTime {
  start: number;
  end: number;
}

export type SkipSource = 'chapters' | 'aniskip';

export interface SkipTimes {
  op?: SkipTime;
  ed?: SkipTime;
  source?: SkipSource;
}

interface EpisodeSkipFields {
  episodeNumber: number;
  opStart?: number;
  opEnd?: number;
  edStart?: number;
  edEnd?: number;
  skipFetched?: boolean;
  skipSource?: SkipSource;
}

async function persistSkipTimes(
  seriesId: string,
  episodeNumber: number,
  times: SkipTimes,
  source: SkipSource,
): Promise<void> {
  try {
    await metadataHandler.transaction(async (meta) => {
      const series = meta[seriesId] as { episodes?: EpisodeSkipFields[] } | undefined;
      if (!series?.episodes) return { result: undefined, updated: null };
      const ep = series.episodes.find((e) => e.episodeNumber === episodeNumber);
      if (!ep) return { result: undefined, updated: null };
      ep.opStart = times.op?.start;
      ep.opEnd = times.op?.end;
      ep.edStart = times.ed?.start;
      ep.edEnd = times.ed?.end;
      ep.skipFetched = true;
      ep.skipSource = source;
      return { result: undefined, updated: meta };
    });
  } catch (err) {
    logger.warn('metadata', `Skip-times cache write failed: ${(err as Error).message}`);
  }
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
   * Resolve intro/outro skip times for one episode. Tries embedded chapter
   * markers (Intro/Outro/Credits/etc.) first when a filePath is provided —
   * those are local, free, and authoritative when present. Falls back to
   * the AniSkip community database keyed on MAL id.
   *
   * Persists the result on the matching episode in metadata.json along with
   * `skipSource` so subsequent plays know whether the cached values came
   * from the file's own chapters or from the crowd-sourced API.
   */
  async fetchAndCache(
    seriesId: string,
    malId: number,
    episodeNumber: number,
    episodeLength: number,
    filePath?: string,
  ): Promise<SkipTimes> {
    // 1. Local chapters. Cheap (~50ms ffprobe) and trumps AniSkip when
    //    the encoder did the work for us.
    if (filePath) {
      const chapters = await probeChapters(filePath);
      const chapterTimes = classifyChapters(chapters);
      if (chapterTimes.op || chapterTimes.ed) {
        const summary = [
          chapterTimes.op ? `op=[${chapterTimes.op.start.toFixed(0)},${chapterTimes.op.end.toFixed(0)}]` : null,
          chapterTimes.ed ? `ed=[${chapterTimes.ed.start.toFixed(0)},${chapterTimes.ed.end.toFixed(0)}]` : null,
        ].filter(Boolean).join(' ');
        logger.info('metadata', `Chapter skip ep=${episodeNumber} ${summary}`);
        await persistSkipTimes(seriesId, episodeNumber, chapterTimes, 'chapters');
        return { ...chapterTimes, source: 'chapters' };
      }
    }

    // 2. AniSkip community lookup.
    if (!malId || !episodeNumber || !episodeLength || episodeLength <= 0) {
      return {};
    }
    const url = `https://api.aniskip.com/v2/skip-times/${malId}/${episodeNumber}?types[]=op&types[]=ed&episodeLength=${Math.round(episodeLength)}`;
    const times: SkipTimes = {};
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

    await persistSkipTimes(seriesId, episodeNumber, times, 'aniskip');
    return { ...times, source: 'aniskip' };
  },
};

export default aniSkipHandler;
