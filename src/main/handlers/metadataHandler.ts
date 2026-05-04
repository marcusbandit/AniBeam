import { readFile, writeFile, mkdir } from 'fs/promises';
import { join } from 'path';
import { app } from 'electron';
import { logger } from '../services/logger';

function getMetadataPath(): string {
  const userDataPath = app.getPath('userData');
  return join(userDataPath, 'metadata.json');
}

async function ensureDataDirectory(): Promise<void> {
  const userDataPath = app.getPath('userData');
  try {
    await mkdir(userDataPath, { recursive: true });
  } catch (error) {
    if ((error as NodeJS.ErrnoException).code !== 'EEXIST') {
      throw error;
    }
  }
}

const metadataHandler = {
  async loadMetadata(): Promise<Record<string, unknown>> {
    try {
      await ensureDataDirectory();
      const metadataPath = getMetadataPath();
      const data = await readFile(metadataPath, 'utf-8');
      return JSON.parse(data) as Record<string, unknown>;
    } catch (error) {
      if ((error as NodeJS.ErrnoException).code === 'ENOENT') {
        return {};
      }
      logger.error('metadata', 'Error loading metadata');
      return {};
    }
  },

  async saveMetadata(metadata: Record<string, unknown>): Promise<boolean> {
    try {
      await ensureDataDirectory();
      const metadataPath = getMetadataPath();
      
      // Clean up metadata before saving: remove entries with 0 fileEpisodes
      const cleanedMetadata: Record<string, unknown> = {};
      for (const [seriesId, seriesData] of Object.entries(metadata)) {
        const data = seriesData as { fileEpisodes?: unknown[] };
        const fileEpisodes = data.fileEpisodes || [];
        
        // Only keep entries that have at least one file episode
        if (fileEpisodes.length > 0) {
          cleanedMetadata[seriesId] = seriesData;
        }
      }
      
      await writeFile(metadataPath, JSON.stringify(cleanedMetadata, null, 2), 'utf-8');
      return true;
    } catch (error) {
      logger.error('metadata', 'Error saving metadata');
      throw error;
    }
  },

  async updateSeriesMetadata(seriesId: string, seriesData: Record<string, unknown>): Promise<boolean> {
    try {
      const metadata = await this.loadMetadata();
      const existingSeries = metadata[seriesId] as Record<string, unknown> | undefined;
      metadata[seriesId] = {
        ...existingSeries,
        ...seriesData,
      };
      const metadataPath = getMetadataPath();
      await writeFile(metadataPath, JSON.stringify(metadata, null, 2), 'utf-8');
      return true;
    } catch (error) {
      logger.error('metadata', 'Error updating series metadata');
      throw error;
    }
  },

  async getSeriesMetadata(seriesId: string): Promise<Record<string, unknown> | null> {
    try {
      const metadata = await this.loadMetadata();
      return (metadata[seriesId] as Record<string, unknown>) || null;
    } catch (error) {
      logger.error('metadata', 'Error getting series metadata');
      return null;
    }
  },

  async deleteSeriesMetadata(seriesId: string): Promise<boolean> {
    try {
      const metadata = await this.loadMetadata();
      delete metadata[seriesId];
      const metadataPath = getMetadataPath();
      await writeFile(metadataPath, JSON.stringify(metadata, null, 2), 'utf-8');
      return true;
    } catch (error) {
      logger.error('metadata', 'Error deleting series metadata');
      throw error;
    }
  },
};

export default metadataHandler;
