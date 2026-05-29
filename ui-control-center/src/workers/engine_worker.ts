// Engine Worker — Background Compute Thread
//
// This worker:
// 1. Initializes a Wasm module for columnar data processing
// 2. Enters an Atomics.wait() sleep loop waiting for task signals
// 3. On wake-up, reads task metadata from SharedArrayBuffer
// 4. Executes the task (ingest/query/parse/sort)
// 5. Writes results back to SharedArrayBuffer
// 6. Signals completion via Atomics.store() + postMessage()

const CONTROL_OFFSET = 0;
const TASK_TYPE_OFFSET = 4;
const DATA_OFFSET_FIELD = 8;
const DATA_LEN_FIELD = 12;
const RESULT_OFFSET_FIELD = 16;
const RESULT_LEN_FIELD = 20;

const TASK_INGEST = 1;
const TASK_QUERY = 2;

let workerId = -1;
let sharedBuffer: SharedArrayBuffer | null = null;
let controlView: Int32Array | null = null;
let wasmModule: any = null; // WebAssembly.Instance

self.onmessage = async (event: MessageEvent) => {
  const { type, sharedBuffer: sb, workerId: wid } = event.data;

  if (type === 'init') {
    workerId = wid;
    sharedBuffer = sb;
    controlView = new Int32Array(sharedBuffer!, 0, 6);

    // Initialize Wasm module
    // In production: const { WasmEngine } = await import('columnar-engine');
    // wasmModule = WasmEngine.new(256);
    console.log(`[Worker ${workerId}] Initialized. Shared memory: ${(sb.byteLength / 1024 / 1024).toFixed(0)}MB`);

    // Enter the atomic sleep loop
    runLoop();
  }
};

function runLoop(): void {
  while (true) {
    // Sleep until the main thread signals a task
    // Atomics.wait() parks the thread efficiently — zero CPU while idle
    const result = Atomics.wait(controlView!, 0, 0, 100);

    if (result === 'ok' || result === 'not-equal') {
      const flags = Atomics.load(controlView!, 0);
      const targetWorker = (flags >> 2) & 0x3F;

      // Only process if this task is for this worker
      if (targetWorker !== workerId) {
        continue;
      }

      const taskType = Atomics.load(controlView!, 1);
      const dataOffset = Atomics.load(controlView!, 2);
      const dataLength = Atomics.load(controlView!, 3);

      // Clear the task flag
      Atomics.store(controlView!, 0, 0);

      // Execute the task
      switch (taskType) {
        case TASK_INGEST:
          executeIngest(dataOffset, dataLength);
          break;
        case TASK_QUERY:
          executeQuery(dataOffset, dataLength);
          break;
        default:
          console.warn(`[Worker ${workerId}] Unknown task type: ${taskType}`);
      }
    }
  }
}

function executeIngest(dataOffset: number, dataLength: number): void {
  // Read raw bytes from shared buffer
  // const ptr = new Uint8Array(sharedBuffer!, dataOffset, dataLength);
  // wasmModule.ingest(ptr, dataLength);

  // In simulation: log the operation
  console.log(`[Worker ${workerId}] INGEST: ${dataLength}B at offset ${dataOffset}`);

  // Simulate processing time (100-400μs)
  const start = performance.now();
  while (performance.now() - start < 0.3) { /* spin — simulated wasm execution */ }

  // Signal completion
  Atomics.store(controlView!, 0, 0);
}

function executeQuery(dataOffset: number, dataLength: number): void {
  // Read query parameters from shared buffer
  // const params = new Float64Array(sharedBuffer!, dataOffset, 2);
  // const result = wasmModule.query_lat_range(params[0], params[1]);

  // Allocate result space in shared buffer
  const resultOffset = dataOffset + dataLength + 256;
  const resultView = new Uint32Array(sharedBuffer!, resultOffset, 128);

  // Simulate result data (matched row indices)
  const matchCount = Math.floor(Math.random() * 500) + 100;
  for (let i = 0; i < matchCount; i++) {
    resultView[i] = Math.floor(Math.random() * 65536);
  }

  // Store result metadata
  Atomics.store(controlView!, 4, resultOffset);
  Atomics.store(controlView!, 5, matchCount);

  console.log(`[Worker ${workerId}] QUERY: ${matchCount} matches`);

  // Signal completion via postMessage (pass memory pointers, not data)
  self.postMessage({
    taskId: Atomics.load(controlView!, 2),
    resultOffset,
    resultLength: matchCount,
  });
}
