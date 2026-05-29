declare module "*.css";

declare module "*/workers/engine_pool" {
  export class EngineWorkerPool {
    constructor(workerCount?: number);
    ingest(data: Uint8Array): Promise<void>;
    query(minLat: number, maxLat: number): Promise<Uint32Array>;
    shutdown(): void;
    get activeWorkers(): number;
    get memoryUsed(): number;
  }
  export const TASK_INGEST: number;
  export const TASK_QUERY: number;
  export const TASK_PARSE: number;
  export const TASK_SORT: number;
}

declare module "*/workers/engine_worker" {}
