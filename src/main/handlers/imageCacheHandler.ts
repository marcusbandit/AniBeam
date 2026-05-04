import { readFile, writeFile, mkdir, access, readdir, rm, stat } from 'fs/promises';
import { join, extname } from 'path';
import { app } from 'electron';
import { createHash } from 'crypto';
import { initProgress, updateProgress } from '../utils/debugUtils';
import { logger } from '../services/logger';

interface CacheEntry {
  originalUrl: string;
  localPath: string;
  cachedAt: string;
}

interface CacheIndex {
  [url: string]: CacheEntry;
}

function getImageCachePath(): string {
  const userDataPath = app.getPath('userData');
  return join(userDataPath, 'image-cache');
}

function getCacheIndexPath(): string {
  return join(getImageCachePath(), 'cache-index.json');
}

async function ensureCacheDirectory(): Promise<void> {
  const cachePath = getImageCachePath();
  try {
    await mkdir(cachePath, { recursive: true });
  } catch (error) {
    if ((error as NodeJS.ErrnoException).code !== 'EEXIST') {
      throw error;
    }
  }
}

async function loadCacheIndex(): Promise<CacheIndex> {
  try {
    const indexPath = getCacheIndexPath();
    const data = await readFile(indexPath, 'utf-8');
    return JSON.parse(data) as CacheIndex;
  } catch (error) {
    if ((error as NodeJS.ErrnoException).code === 'ENOENT') {
      return {};
    }
    logger.error('image', 'Error loading cache index');
    return {};
  }
}

async function saveCacheIndex(index: CacheIndex): Promise<void> {
  try {
    const indexPath = getCacheIndexPath();
    await writeFile(indexPath, JSON.stringify(index, null, 2), 'utf-8');
  } catch (error) {
    logger.error('image', 'Error saving cache index');
  }
}

function generateCacheFilename(url: string): string {
  // Create a hash of the URL for unique filename
  const hash = createHash('md5').update(url).digest('hex');

  // Try to extract extension from URL
  let ext = '.jpg'; // default
  try {
    const urlObj = new URL(url);
    const pathExt = extname(urlObj.pathname);
    if (pathExt && ['.jpg', '.jpeg', '.png', '.gif', '.webp', '.avif'].includes(pathExt.toLowerCase())) {
      ext = pathExt.toLowerCase();
    }
  } catch {
    // Use default extension
  }

  return `${hash}${ext}`;
}

async function fileExists(path: string): Promise<boolean> {
  try {
    await access(path);
    return true;
  } catch {
    return false;
  }
}

const imageCacheHandler = {
  /**
   * Download and cache an image from a URL
   * Returns the local file path or null if download fails
   */
  async cacheImage(imageUrl: string): Promise<string | null> {
    if (!imageUrl) return null;

    try {
      await ensureCacheDirectory();
      const cacheIndex = await loadCacheIndex();

      // Check if already cached
      if (cacheIndex[imageUrl]) {
        const localPath = cacheIndex[imageUrl].localPath;
        if (await fileExists(localPath)) {
          const filename = generateCacheFilename(imageUrl);
          updateProgress('📷 Image caching', filename);
          return localPath;
        }
        // File doesn't exist anymore, remove from index
        delete cacheIndex[imageUrl];
      }

      // Download the image
      const response = await fetch(imageUrl, {
        headers: {
          'User-Agent': 'AniBeam Media Server/1.0',
          'Accept': 'image/*',
        },
      });

      if (!response.ok) {
        logger.error('image', `Failed to download image (${response.status})`, { file: imageUrl });
        return null;
      }

      const filename = generateCacheFilename(imageUrl);
      const localPath = join(getImageCachePath(), filename);

      // Get the image as a buffer and write to file
      const arrayBuffer = await response.arrayBuffer();
      await writeFile(localPath, Buffer.from(arrayBuffer));

      // Update cache index
      cacheIndex[imageUrl] = {
        originalUrl: imageUrl,
        localPath,
        cachedAt: new Date().toISOString(),
      };
      await saveCacheIndex(cacheIndex);

      updateProgress('📷 Image caching', filename);
      return localPath;
    } catch (error) {
      logger.error('image', `Error caching image`, { file: imageUrl });
      return null;
    }
  },

  /**
   * Cache multiple images in parallel with rate limiting
   */
  async cacheImages(urls: (string | null | undefined)[]): Promise<Map<string, string | null>> {
    const results = new Map<string, string | null>();
    const validUrls = urls.filter((url): url is string => !!url);
    if (validUrls.length > 0) {
      initProgress('📷 Image caching', validUrls.length);
    }

    // Process in batches of 5 to avoid overwhelming the network
    const batchSize = 5;
    for (let i = 0; i < validUrls.length; i += batchSize) {
      const batch = validUrls.slice(i, i + batchSize);
      const batchResults = await Promise.all(
        batch.map(async (url) => {
          const localPath = await this.cacheImage(url);
          return { url, localPath };
        })
      );

      for (const { url, localPath } of batchResults) {
        results.set(url, localPath);
      }

      // Small delay between batches
      if (i + batchSize < validUrls.length) {
        await new Promise(resolve => setTimeout(resolve, 100));
      }
    }

    return results;
  },

  /**
   * Get the local path for a cached image URL
   * Returns null if not cached
   */
  async getCachedPath(imageUrl: string): Promise<string | null> {
    if (!imageUrl) return null;

    try {
      const cacheIndex = await loadCacheIndex();
      const entry = cacheIndex[imageUrl];

      if (entry && await fileExists(entry.localPath)) {
        return entry.localPath;
      }

      return null;
    } catch (error) {
      logger.error('image', 'Error getting cached path');
      return null;
    }
  },

  /**
   * Check if an image URL is cached
   */
  async isCached(imageUrl: string): Promise<boolean> {
    return (await this.getCachedPath(imageUrl)) !== null;
  },

  /**
   * Convert a URL to a local path, caching if necessary
   * Returns the media:// URL for local files
   */
  async getLocalUrl(imageUrl: string | null | undefined): Promise<string | null> {
    if (!imageUrl) return null;

    // Check if already a local path
    if (imageUrl.startsWith('/') || imageUrl.startsWith('media://')) {
      return imageUrl;
    }

    const localPath = await this.cacheImage(imageUrl);
    if (localPath) {
      return `media://${encodeURIComponent(localPath)}`;
    }

    // Fallback to original URL if caching fails
    return imageUrl;
  },

  /**
   * Clear all cached images
   */
  async clearCache(): Promise<void> {
    try {
      const cachePath = getImageCachePath();
      await rm(cachePath, { recursive: true, force: true });
      await ensureCacheDirectory();
      await saveCacheIndex({});
      logger.info('image', 'Image cache cleared');
    } catch (error) {
      logger.error('image', 'Error clearing image cache');
      throw error;
    }
  },

  /**
   * Get cache statistics
   */
  async getCacheStats(): Promise<{ count: number; sizeBytes: number }> {
    try {
      const cachePath = getImageCachePath();
      const files = await readdir(cachePath);

      let totalSize = 0;
      let count = 0;

      for (const file of files) {
        if (file === 'cache-index.json') continue;

        try {
          const filePath = join(cachePath, file);
          const stats = await stat(filePath);
          totalSize += stats.size;
          count++;
        } catch {
          // Skip files that can't be accessed
        }
      }

      return { count, sizeBytes: totalSize };
    } catch {
      return { count: 0, sizeBytes: 0 };
    }
  },

  /**
   * Delete cached images for a specific series
   */
  async deleteSeriesImages(seriesData: {
    poster?: string | null;
    banner?: string | null;
    posterLocal?: string | null;
    bannerLocal?: string | null;
    episodes?: Array<{
      thumbnail?: string | null;
      thumbnailLocal?: string | null;
    }>;
  }): Promise<void> {
    try {
      const cacheIndex = await loadCacheIndex();
      const imageUrls: (string | null | undefined)[] = [
        seriesData.poster,
        seriesData.banner,
      ];

      // Add episode thumbnails
      if (seriesData.episodes) {
        for (const ep of seriesData.episodes) {
          if (ep.thumbnail) {
            imageUrls.push(ep.thumbnail);
          }
        }
      }

      let deletedCount = 0;
      for (const url of imageUrls) {
        if (!url) continue;

        const entry = cacheIndex[url];
        if (entry) {
          try {
            // Delete the file if it exists
            if (await fileExists(entry.localPath)) {
              await rm(entry.localPath, { force: true });
              deletedCount++;
            }
            // Remove from index
            delete cacheIndex[url];
          } catch (error) {
            logger.error('image', `Error deleting cached image`, { file: entry.localPath });
          }
        }
      }

      // Also check local paths directly
      const localPaths: (string | null | undefined)[] = [
        seriesData.posterLocal,
        seriesData.bannerLocal,
      ];

      if (seriesData.episodes) {
        for (const ep of seriesData.episodes) {
          if (ep.thumbnailLocal) {
            localPaths.push(ep.thumbnailLocal);
          }
        }
      }

      for (const localPath of localPaths) {
        if (!localPath) continue;

        // Extract actual file path from media:// URL if needed
        const actualPath = localPath.startsWith('media://')
          ? decodeURIComponent(localPath.replace('media://', ''))
          : localPath;

        try {
          if (await fileExists(actualPath)) {
            await rm(actualPath, { force: true });
            deletedCount++;
          }
        } catch (error) {
          logger.error('image', `Error deleting local image`, { file: actualPath });
        }

        // Remove from cache index if found
        for (const [url, entry] of Object.entries(cacheIndex)) {
          if (entry.localPath === actualPath) {
            delete cacheIndex[url];
            break;
          }
        }
      }

      // Save updated cache index
      await saveCacheIndex(cacheIndex);
      logger.info('image', `Deleted ${deletedCount} cached images for series`);
    } catch (error) {
      logger.error('image', 'Error deleting series images');
      throw error;
    }
  },

  /**
   * Get the base cache directory path
   */
  getCachePath(): string {
    return getImageCachePath();
  },
};

export default imageCacheHandler;

