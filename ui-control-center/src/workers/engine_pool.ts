// Engine Worker Pool — Multi-Threaded Browser Coordination Layer
// 
// Architecture: 1 Main UI Thread + N Background Compute Workers
//
// Data Flow:
//   Main Thread                           Worker Pool
//   ┌──────────────┐     SharedArrayBuffer    ┌──────────────────────────┐
//   │ EnginePool   │ ◄─────────────────────► │ W1: Wasm Columnar Engine  │
//   │ .dispatch()  │      atomics.wait()      │ W2: Wasm Columnar Engine  │
//   │ .query()     │      atomics.notify()    │ W3: Wasm Columnar Engine  │
//   │ .shutdown()  │                          │ W4: Wasm Columnar Engine  │
//   └──────────────┘                          └──────────────────────────┘
//
// Memory Layout (SharedArrayBuffer, 128MB):
//   Offset 0-3:       Control flags (Int32Array[0])
//     Bit 0: Task ready flag
//     Bit 1: Worker completion flag
//     Bits 2-7: Target worker ID (round-robin)
//   Offset 4-7:       Task type (Int32Array[1])
//     1 = INGEST, 2 = QUERY, 3 = PARSE, 4 = SORT
//   Offset 8-11:      Data byte offset in buffer (Int32Array[2])
//   Offset 12-15:     Data byte length (Int32Array[3])
//   Offset 16-19:     Result byte offset (Int32Array[4])
//   Offset 20-23:     Result byte length (Int32Array[5])
//   Offset 24+:       Raw binary data payload

const SHARED_MEMORY_SIZE = 128 * 1024 * 1024; // 128MB
const CONTROL_OFFSET = 0;
const TASK_TYPE_OFFSET = 4;
const DATA_OFFSET_FIELD = 8;
const DATA_LEN_FIELD = 12;
const RESULT_OFFSET_FIELD = 16;
const RESULT_LEN_FIELD = 20;
const DATA_START = 24;

const TASK_INGEST = 1;
const TASK_QUERY = 2;
const TASK_PARSE = 3;
const TASK_SORT = 4;

const FLAG_TASK_READY = 0x1;
const FLAG_WORKER_DONE = 0x2;

class EngineWorkerPool {
  private workers: Worker[];
  private sharedBuffer: SharedArrayBuffer;
  private controlView: Int32Array;
  private roundRobinIndex: number;
  private pendingTasks: Map<number, (result: any) => void>;
  private taskIdCounter: number;

  constructor(workerCount: number = 4) {
    this.workers = [];
    this.roundRobinIndex = 0;
    this.pendingTasks = new Map();
    this.taskIdCounter = 0;

    // Allocate the shared memory buffer
    this.sharedBuffer = new SharedArrayBuffer(SHARED_MEMORY_SIZE);
    this.controlView = new Int32Array(this.sharedBuffer, CONTROL_OFFSET, 6);

    // Initialize control fields
    Atomics.store(this.controlView, 0, 0);
    Atomics.store(this.controlView, 1, 0);

    // Spawn worker pool
    for (let i = 0; i < workerCount; i++) {
      const worker = new Worker(
        new URL('./engine_worker.ts', import.meta.url),
        { type: 'module' }
      );

      // Send the shared buffer to each worker
      worker.postMessage({
        type: 'init',
        sharedBuffer: this.sharedBuffer,
        workerId: i,
      });

      this.workers.push(worker);

      // Listen for completion signals
      worker.onmessage = (event) => {
        const { taskId, resultOffset, resultLength } = event.data;
        const resolve = this.pendingTasks.get(taskId);
        if (resolve) {
          // Extract result pointers from shared memory
          const resultView = new Uint32Array(
            this.sharedBuffer,
            resultOffset,
            resultLength
          );
          resolve(Array.from(resultView));
          this.pendingTasks.delete(taskId);
        }
      };

      worker.onerror = (error) => {
        console.error(`[Worker ${i}] Crashed:`, error);
        this.restartWorker(i);
      };
    }
  }

  // Ingest raw binary data into the shared buffer for processing
  async ingest(data: Uint8Array): Promise<void> {
    const offset = this.allocateData(data.byteLength);
    new Uint8Array(this.sharedBuffer, offset, data.byteLength).set(data);

    return this.dispatchTask(TASK_INGEST, offset, data.byteLength);
  }

  // Execute a vectorized query on a specific worker
  async query(minLat: number, maxLat: number): Promise<Uint32Array> {
    const params = new Float64Array([minLat, maxLat]);
    const offset = this.allocateData(params.byteLength);
    new Float64Array(this.sharedBuffer, offset, 2).set(params);

    const result = await this.dispatchTask(TASK_QUERY, offset, params.byteLength);
    return new Uint32Array(result as number[]);
  }

  private allocateData(size: number): number {
    // Simple bump allocator — in production, use a proper ring buffer
    const allocationOffset = DATA_START + (this.taskIdCounter % 1024) * 65536;
    return allocationOffset;
  }

  private async dispatchTask(
    taskType: number,
    dataOffset: number,
    dataLength: number
  ): Promise<any> {
    const taskId = this.taskIdCounter++;
    const workerId = this.roundRobinIndex % this.workers.length;
    this.roundRobinIndex++;

    return new Promise((resolve) => {
      this.pendingTasks.set(taskId, resolve);

      // Write task metadata to shared control ring
      Atomics.store(this.controlView, 0, FLAG_TASK_READY | (workerId << 2));
      Atomics.store(this.controlView, 1, taskType);
      Atomics.store(this.controlView, 2, dataOffset);
      Atomics.store(this.controlView, 3, dataLength);

      // Wake the target worker
      Atomics.notify(this.controlView, 0, 1);
    });
  }

  private restartWorker(workerId: number): void {
    const oldWorker = this.workers[workerId];
    oldWorker.terminate();

    const newWorker = new Worker(
      new URL('./engine_worker.ts', import.meta.url),
      { type: 'module' }
    );

    newWorker.postMessage({
      type: 'init',
      sharedBuffer: this.sharedBuffer,
      workerId,
    });

    newWorker.onmessage = oldWorker.onmessage;
    newWorker.onerror = oldWorker.onerror;

    this.workers[workerId] = newWorker;
  }

  shutdown(): void {
    for (const worker of this.workers) {
      worker.terminate();
    }
    this.workers = [];
  }

  get activeWorkers(): number {
    return this.workers.length;
  }

  get memoryUsed(): number {
    return SHARED_MEMORY_SIZE;
  }
}

export { EngineWorkerPool, TASK_INGEST, TASK_QUERY, TASK_PARSE, TASK_SORT };
