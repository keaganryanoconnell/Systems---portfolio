"use client";

import { createContext, useContext, useState, useEffect, useCallback, useRef } from "react";

interface WorkerContextValue {
  mode: "sim" | "live";
  toggleMode: () => void;
  sharedBufferAvailable: boolean;
  activeWorkers: number;
  workerStates: string[];
  heapUsed: number;
  heapMax: number;
  liveConnecting: boolean;
}

const WorkerContext = createContext<WorkerContextValue>({
  mode: "sim",
  toggleMode: () => {},
  sharedBufferAvailable: false,
  activeWorkers: 0,
  workerStates: [],
  heapUsed: 180 * 1024 * 1024,
  heapMax: 256 * 1024 * 1024,
  liveConnecting: false,
});

export function useWorkerEngine() {
  return useContext(WorkerContext);
}

function isSharedArrayBufferAvailable(): boolean {
  try {
    return typeof SharedArrayBuffer !== "undefined" && typeof Atomics !== "undefined";
  } catch {
    return false;
  }
}

export function EngineWorkerProvider({ children }: { children: React.ReactNode }) {
  const [mode, setMode] = useState<"sim" | "live">("sim");
  const [sharedBufferAvailable] = useState(() => isSharedArrayBufferAvailable());
  const [activeWorkers, setActiveWorkers] = useState(0);
  const [workerStates, setWorkerStates] = useState<string[]>(["IDLE", "IDLE", "IDLE", "IDLE"]);
  const [heapUsed, setHeapUsed] = useState(180 * 1024 * 1024);
  const [heapMax] = useState(256 * 1024 * 1024);
  const [liveConnecting, setLiveConnecting] = useState(false);
  const poolRef = useRef<any>(null);
  const intervalRef = useRef<ReturnType<typeof setInterval> | null>(null);

  const toggleMode = useCallback(() => {
    setMode((prev) => {
      if (prev === "sim") {
        if (!isSharedArrayBufferAvailable()) {
          return "sim";
        }
        setLiveConnecting(true);
        return "live";
      }
      return "sim";
    });
  }, []);

  useEffect(() => {
    if (mode === "live" && sharedBufferAvailable && !poolRef.current) {
      const initPool = async () => {
        try {
          const { EngineWorkerPool } = await import("../../workers/engine_pool");
          const pool = new EngineWorkerPool(4);
          poolRef.current = pool;
          setActiveWorkers(4);
          setLiveConnecting(false);

          intervalRef.current = setInterval(() => {
            if (!poolRef.current) return;
            setWorkerStates(["QUERY", "PARSE", "IDLE", "QUERY"]);
            setHeapUsed((prev) => {
              const delta = Math.random() > 0.96
                ? -(Math.random() * 30 + 5) * 1024 * 1024
                : Math.random() * 3 * 1024 * 1024;
              return Math.max(10 * 1024 * 1024, Math.min(256 * 1024 * 1024, prev + delta));
            });
          }, 1000);
        } catch (err) {
          console.warn("[EngineWorkerProvider] Worker init failed, staying in SIM mode:", err);
          setMode("sim");
          setLiveConnecting(false);
        }
      };
      initPool();
    }

    return () => {
      if (intervalRef.current) clearInterval(intervalRef.current);
    };
  }, [mode, sharedBufferAvailable]);

  useEffect(() => {
    if (mode === "sim") {
      if (poolRef.current) {
        poolRef.current.shutdown();
        poolRef.current = null;
      }
      setActiveWorkers(0);
      setLiveConnecting(false);

      const simInterval = setInterval(() => {
        setHeapUsed((prev) => {
          const delta = Math.random() > 0.97
            ? -(Math.random() * 20 + 5) * 1024 * 1024
            : Math.random() * 2 * 1024 * 1024;
          return Math.max(10 * 1024 * 1024, Math.min(256 * 1024 * 1024, prev + delta));
        });
        setWorkerStates((prev) =>
          prev.map(() => {
            const r = Math.random();
            if (r < 0.3) return "IDLE";
            if (r < 0.6) return "QUERY";
            if (r < 0.85) return "PARSE";
            return "IDLE";
          })
        );
      }, 1000);
      return () => clearInterval(simInterval);
    }
  }, [mode]);

  return (
    <WorkerContext.Provider
      value={{
        mode,
        toggleMode,
        sharedBufferAvailable,
        activeWorkers,
        workerStates,
        heapUsed,
        heapMax,
        liveConnecting,
      }}
    >
      {children}
    </WorkerContext.Provider>
  );
}
