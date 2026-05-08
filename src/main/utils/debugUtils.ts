import cliProgress from 'cli-progress';
import { logger } from '../services/logger';
import type { LogStage } from '../../shared/logTypes';

function stageFor(header: string): LogStage {
  const h = header.toLowerCase();
  if (h.includes('thumb')) return 'thumbnail';
  if (h.includes('image') || h.includes('cache')) return 'image';
  if (h.includes('metadata') || h.includes('media')) return 'metadata';
  return 'system';
}

// cli-progress writes ANSI escape sequences to stdout. In a packaged
// Electron app launched from a .desktop entry, stdout is not connected
// to a terminal and the first write that hits a closed pipe crashes the
// main process with EPIPE. Skip the visual bar entirely when there's no
// TTY — the structured `logger` output downstream still goes through.
const TTY_AVAILABLE = !!process.stdout.isTTY;

const BAR_SIZE = 35;

interface ProgressBarInstance {
    bar: cliProgress.SingleBar;
    current: number;
    total: number;
    header: string;
}

// Store multiple progress bars by their header/ID
const progressBars = new Map<string, ProgressBarInstance>();

function createProgressBar(header: string, total: number = 0): ProgressBarInstance {
    const bar = new cliProgress.SingleBar({
        format: (_options, params, payload) => {
            const barSize = BAR_SIZE;
            // Use params.progress if available (0-1), otherwise calculate
            const progress = params.progress !== undefined ? params.progress :
                (params.total > 0 ? params.value / params.total : params.value > 0 ? 1 : 0);
            const filled = Math.min(Math.max(0, Math.round(progress * barSize)), barSize);

            const barComplete = '#'.repeat(barSize);
            const barIncomplete = '-'.repeat(barSize);

            const barDisplay = '\x1b[32m' + barComplete.substring(0, filled) + '\x1b[0m' +
                '\x1b[90m' + barIncomplete.substring(0, barSize - filled) + '\x1b[0m';

            const headerText = (payload as { header?: string }).header || header;
            const displayTotal = params.total > 0 ? params.total : params.value;

            return (
                `\x1b[36m${headerText}\x1b[0m: [${barDisplay}] ` +
                `\x1b[35m${params.value}\x1b[0m/\x1b[35m${displayTotal}\x1b[0m\n`
            );
        },
        hideCursor: true,
        clearOnComplete: false,
        stopOnComplete: false,
        barsize: BAR_SIZE,
    });

    bar.start(total > 0 ? total : 1, 0, { header });

    return {
        bar,
        current: 0,
        total: total > 0 ? total : 1,
        header,
    };
}

/**
 * Initialize a progress bar with a header/title
 * Resets any existing progress bar with the same header
 * @param header The title/header for the progress bar
 * @param total The total number of items (0 if unknown, will be updated dynamically)
 */
export function initProgress(header: string, total: number = 0): void {
    if (!TTY_AVAILABLE) return;
    // Stop and remove existing progress bar if it exists (reset for new operation)
    if (progressBars.has(header)) {
        const instance = progressBars.get(header)!;
        instance.bar.stop();
        progressBars.delete(header);
    }

    const instance = createProgressBar(header, total);
    progressBars.set(header, instance);
}

/**
 * Update a progress bar by incrementing the current count
 * @param header The header/title of the progress bar to update
 * @param filename Optional filename to display
 */
export function updateProgress(header: string, filename?: string): void {
    if (!TTY_AVAILABLE) {
        // No bar without a TTY, but the structured logger should still see
        // the event so the activity drawer in the renderer stays populated.
        logger.info(stageFor(header), filename ? `${header}: ${filename}` : header);
        return;
    }
    const instance = progressBars.get(header);

    if (!instance) {
        // Auto-initialize if not exists (for backward compatibility)
        initProgress(header, 0);
        const newInstance = progressBars.get(header)!;
        newInstance.current++;
        newInstance.total = Math.max(newInstance.total, newInstance.current);
        newInstance.bar.setTotal(newInstance.total);
        newInstance.bar.update(newInstance.current, { header, filename });
        logger.info(stageFor(header), filename ? `${header}: ${filename}` : header);
        return;
    }

    instance.current++;

    // If total is 0 or current exceeds total, update total
    if (instance.total === 0 || instance.current > instance.total) {
        instance.total = instance.current;
        instance.bar.setTotal(instance.total);
    }

    instance.bar.update(instance.current, { header, filename });
    logger.info(stageFor(header), filename ? `${header}: ${filename}` : header);
}

/**
 * Set the total for a progress bar (useful when you know the total upfront)
 * @param header The header/title of the progress bar
 * @param total The total number of items
 */
export function setProgressTotal(header: string, total: number): void {
    if (!TTY_AVAILABLE) return;
    const instance = progressBars.get(header);

    if (!instance) {
        initProgress(header, total);
        return;
    }

    instance.total = total;
    instance.bar.setTotal(total);
    instance.bar.update(instance.current, { header });
}

/**
 * Reset a progress bar (set current to 0)
 * @param header The header/title of the progress bar
 */
export function resetProgress(header: string): void {
    if (!TTY_AVAILABLE) return;
    const instance = progressBars.get(header);

    if (instance) {
        instance.current = 0;
        instance.bar.update(0, { header });
    }
}

/**
 * Stop and remove a progress bar
 * @param header The header/title of the progress bar
 */
export function stopProgress(header: string): void {
    if (!TTY_AVAILABLE) return;
    const instance = progressBars.get(header);

    if (instance) {
        instance.bar.stop();
        progressBars.delete(header);
    }
}

// Legacy functions for backward compatibility
export function initMediaProgress(): void {
    initProgress('Media requests', 0);
}

export function updateMediaProgress(filePath?: string): void {
    const filename = filePath ? filePath.split('/').pop() || filePath : undefined;
    updateProgress('Media requests', filename);
}
