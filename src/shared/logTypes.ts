export type LogLevel = 'info' | 'warn' | 'error';
export type LogStage =
  | 'folder'
  | 'metadata'
  | 'image'
  | 'thumbnail'
  | 'watch'
  | 'probe'
  | 'system';

export interface LogEvent {
  id: number;
  ts: number;
  level: LogLevel;
  stage: LogStage;
  message: string;
  ctx?: { series?: string; file?: string };
}
