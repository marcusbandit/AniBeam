import { mkdir, access } from 'fs/promises';
import { join, basename } from 'path';
import { app } from 'electron';
import { spawn } from 'child_process';
import { createHash } from 'crypto';
import { initProgress, updateProgress } from '../utils/debugUtils';
import { logger } from '../services/logger';

function getThumbnailCachePath(): string {
  const userDataPath = app.getPath('userData');
  return join(userDataPath, 'thumbnails');
}

async function ensureThumbnailDirectory(): Promise<void> {
  const cachePath = getThumbnailCachePath();
  try {
    await mkdir(cachePath, { recursive: true });
  } catch (error) {
    if ((error as NodeJS.ErrnoException).code !== 'EEXIST') {
      throw error;
    }
  }
}

async function fileExists(path: string): Promise<boolean> {
  try {
    await access(path);
    return true;
  } catch {
    return false;
  }
}

function generateThumbnailFilename(videoPath: string, timestamp: number): string {
  const hash = createHash('md5').update(`${videoPath}_${timestamp}`).digest('hex');
  return `${hash}.jpg`;
}

/**
 * Extract a frame from a video file using ffmpeg
 * @param videoPath Path to the video file
 * @param timestamp Timestamp in seconds (default: 120 = 2 minutes)
 * @returns Path to the generated thumbnail or null if failed
 */
async function extractFrame(videoPath: string, timestamp: number = 120, resetProgress: boolean = false): Promise<string | null> {
  await ensureThumbnailDirectory();
  
  const thumbnailFilename = generateThumbnailFilename(videoPath, timestamp);
  const thumbnailPath = join(getThumbnailCachePath(), thumbnailFilename);
  
  if (resetProgress) {
    initProgress('🎬 Generating thumbnails', 0);
  }
  
  // Check if thumbnail already exists
  if (await fileExists(thumbnailPath)) {
    const filename = basename(videoPath);
    updateProgress('🎬 Generating thumbnails', filename);
    return thumbnailPath;
  }
  
  return new Promise((resolve) => {
    // Track if promise has already resolved to prevent race conditions
    let resolved = false;
    let timeoutId: NodeJS.Timeout | null = null;

    const safeResolve = (value: string | null) => {
      if (!resolved) {
        resolved = true;
        if (timeoutId) {
          clearTimeout(timeoutId);
          timeoutId = null;
        }
        resolve(value);
      }
    };

    // Use ffmpeg to extract a frame
    // -ss: seek to timestamp
    // -i: input file
    // -vframes 1: extract 1 frame
    // -q:v 2: quality (2 is high quality for jpeg)
    // -vf scale: resize to 480px width, maintain aspect ratio
    const ffmpeg = spawn('ffmpeg', [
      '-ss', String(timestamp),
      '-i', videoPath,
      '-vframes', '1',
      '-q:v', '2',
      '-vf', 'scale=480:-1',
      '-y', // Overwrite output file
      thumbnailPath,
    ], {
      stdio: ['ignore', 'ignore', 'ignore'], // Suppress ffmpeg output
    });

    ffmpeg.on('close', async (code) => {
      if (resolved) return; // Already resolved by timeout or error
      
      if (code === 0 && await fileExists(thumbnailPath)) {
        const filename = basename(videoPath);
        updateProgress('🎬 Generating thumbnails', filename);
        safeResolve(thumbnailPath);
      } else {
        // If seeking to 2 min fails (video too short), try 10 seconds
        if (timestamp > 10) {
          const retryResult = await extractFrame(videoPath, 10, false);
          safeResolve(retryResult);
        } else {
          // Still update progress even on failure
          const filename = basename(videoPath);
          updateProgress('🎬 Generating thumbnails', filename);
          safeResolve(null);
        }
      }
    });

    ffmpeg.on('error', (err) => {
      if (resolved) return; // Already resolved
      logger.error('thumbnail', `ffmpeg error: ${err.message}`, { file: videoPath });
      safeResolve(null);
    });

    // Timeout after 30 seconds
    timeoutId = setTimeout(() => {
      if (resolved) return; // Already resolved
      ffmpeg.kill();
      safeResolve(null);
    }, 30000);
  });
}

const thumbnailHandler = {
  /**
   * Generate a thumbnail for an episode from the video file
   * @param videoPath Path to the video file
   * @param timestamp Timestamp in seconds (default: 120 = 2 minutes)
   * @param resetProgressBar Reset the progress bar (use true for first thumbnail in a batch)
   */
  async generateThumbnail(videoPath: string, timestamp: number = 120, resetProgressBar: boolean = true): Promise<string | null> {
    if (!videoPath) return null;
    
    try {
      return await extractFrame(videoPath, timestamp, resetProgressBar);
    } catch (error) {
      logger.error('thumbnail', `Error generating thumbnail`, { file: videoPath });
      return null;
    }
  },

  /**
   * Generate thumbnails for multiple video files
   * Processes sequentially to avoid overwhelming the system
   */
  async generateThumbnails(videoPaths: string[]): Promise<Map<string, string | null>> {
    const results = new Map<string, string | null>();
    
    // Progress bar will be initialized by caller if needed
    for (const videoPath of videoPaths) {
      const thumbnail = await this.generateThumbnail(videoPath);
      results.set(videoPath, thumbnail);
    }
    
    return results;
  },

  /**
   * Get the thumbnail cache directory path
   */
  getCachePath(): string {
    return getThumbnailCachePath();
  },

  /**
   * Check if ffmpeg is available
   */
  async isFFmpegAvailable(): Promise<boolean> {
    return new Promise((resolve) => {
      const ffmpeg = spawn('ffmpeg', ['-version'], {
        stdio: ['ignore', 'ignore', 'ignore'],
      });

      ffmpeg.on('close', (code) => {
        resolve(code === 0);
      });

      ffmpeg.on('error', () => {
        resolve(false);
      });

      setTimeout(() => {
        ffmpeg.kill();
        resolve(false);
      }, 5000);
    });
  },
};

export default thumbnailHandler;
