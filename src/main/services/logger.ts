import { BrowserWindow } from 'electron';

export type { LogLevel, LogStage, LogEvent } from '../../shared/logTypes';
import type { LogLevel, LogStage, LogEvent } from '../../shared/logTypes';

const BUFFER_LIMIT = 5000;
let nextId = 1;
const buffer: LogEvent[] = [];

function broadcast(event: LogEvent): void {
  for (const win of BrowserWindow.getAllWindows()) {
    if (!win.isDestroyed()) {
      win.webContents.send('log:event', event);
    }
  }
}

function record(level: LogLevel, stage: LogStage, message: string, ctx?: LogEvent['ctx']): void {
  const event: LogEvent = { id: nextId++, ts: Date.now(), level, stage, message, ctx };
  buffer.push(event);
  if (buffer.length > BUFFER_LIMIT) buffer.shift();
  const consoleMethod = level === 'error' ? console.error : level === 'warn' ? console.warn : console.log;
  const ctxBit = ctx?.series ? ` [${ctx.series}]` : ctx?.file ? ` [${ctx.file}]` : '';
  consoleMethod(`[${stage}]${ctxBit} ${message}`);
  broadcast(event);
}

export const logger = {
  info(stage: LogStage, message: string, ctx?: LogEvent['ctx']): void {
    record('info', stage, message, ctx);
  },
  warn(stage: LogStage, message: string, ctx?: LogEvent['ctx']): void {
    record('warn', stage, message, ctx);
  },
  error(stage: LogStage, message: string, ctx?: LogEvent['ctx']): void {
    record('error', stage, message, ctx);
  },
  getBuffer(): LogEvent[] {
    return buffer.slice();
  },
  /** Empties the buffer. Does NOT reset the id counter — ids remain monotonic across clears. */
  clear(): void {
    buffer.length = 0;
  },
};
