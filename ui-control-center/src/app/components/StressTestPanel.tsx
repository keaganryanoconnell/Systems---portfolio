"use client";

import { useState, useEffect, useRef, useCallback } from "react";
import { Flame, Zap } from "lucide-react";

function latencyColor(us: number): string {
  if (us < 500) return "#3fb950";
  if (us < 2000) return "#d2991d";
  return "#f85149";
}

function HistogramBar({ value, max, color }: { value: number; max: number; color: string }) {
  const pct = max === 0 ? 0 : Math.min(100, (value / max) * 100);
  return (
    <div className="flex items-center gap-2">
      <div className="flex-1 h-2 bg-bg rounded overflow-hidden">
        <div
          className="h-full rounded transition-all duration-300"
          style={{ width: `${pct}%`, background: color }}
        />
      </div>
      <span className="text-[8px] font-mono text-text-muted w-12 text-right">{value}</span>
    </div>
  );
}

export default function StressTestPanel() {
  const [concurrency, setConcurrency] = useState(8);
  const [queryCount, setQueryCount] = useState(1000);
  const [running, setRunning] = useState(false);
  const [dispatched, setDispatched] = useState(0);
  const [completed, setCompleted] = useState(0);
  const [throughput, setThroughput] = useState(0);
  const [p50, setP50] = useState(0);
  const [p99, setP99] = useState(0);
  const [p999, setP999] = useState(0);
  const [queueDepth, setQueueDepth] = useState(0);
  const [maxQueueDepth, setMaxQueueDepth] = useState(64);
  const [saturated, setSaturated] = useState(false);
  const [histogram, setHistogram] = useState<number[]>([0, 0, 0, 0, 0, 0, 0, 0]);
  const [log, setLog] = useState<string[]>(["READY — Configure load and press FIRE"]);
  const intervalRef = useRef<ReturnType<typeof setInterval> | null>(null);
  const startTimeRef = useRef(0);
  const totalDispatchedRef = useRef(0);
  const totalCompletedRef = useRef(0);

  const buckets = ["<100μs", "100-500μs", "500μs-1ms", "1-5ms", "5-20ms", "20-100ms", "100ms-1s", ">1s"];

  const fire = useCallback(() => {
    if (running) return;
    setRunning(true);
    setDispatched(0);
    setCompleted(0);
    setThroughput(0);
    setP50(0);
    setP99(0);
    setP999(0);
    setQueueDepth(0);
    setSaturated(false);
    setHistogram([0, 0, 0, 0, 0, 0, 0, 0]);
    startTimeRef.current = Date.now();
    totalDispatchedRef.current = 0;
    totalCompletedRef.current = 0;

    setLog(["═══════ STRESS TEST STARTED ═══════", `Concurrency: ${concurrency} · Total: ${queryCount.toLocaleString()} queries`]);

    let remaining = queryCount;
    let inFlight = 0;
    const latencies: number[] = [];

    const recordLatency = (us: number) => {
      latencies.push(us);
      totalCompletedRef.current++;
      setCompleted(totalCompletedRef.current);
      setQueueDepth((p) => Math.max(0, p - 1));

      if (us < 100) setHistogram((h) => { const n = [...h]; n[0]++; return n; });
      else if (us < 500) setHistogram((h) => { const n = [...h]; n[1]++; return n; });
      else if (us < 1000) setHistogram((h) => { const n = [...h]; n[2]++; return n; });
      else if (us < 5000) setHistogram((h) => { const n = [...h]; n[3]++; return n; });
      else if (us < 20000) setHistogram((h) => { const n = [...h]; n[4]++; return n; });
      else if (us < 100000) setHistogram((h) => { const n = [...h]; n[5]++; return n; });
      else if (us < 1000000) setHistogram((h) => { const n = [...h]; n[6]++; return n; });
      else setHistogram((h) => { const n = [...h]; n[7]++; return n; });
    };

    const sendQuery = () => {
      if (remaining <= 0) return;
      remaining--;
      inFlight++;
      totalDispatchedRef.current++;
      setDispatched(totalDispatchedRef.current);
      setQueueDepth((p) => Math.min(p + 1, maxQueueDepth));

      if (inFlight > concurrency * 2) setSaturated(true);

      const delay = Math.floor(Math.random() * 5000) + 100 + (saturated ? Math.random() * 8000 : 0);
      setTimeout(() => {
        recordLatency(delay);
        inFlight--;
        if (remaining > 0 || inFlight > 0) sendQuery();
        else if (totalCompletedRef.current >= queryCount) {
          setRunning(false);
          const elapsed = (Date.now() - startTimeRef.current) / 1000;
          latencies.sort((a, b) => a - b);
          const p50v = latencies[Math.floor(latencies.length * 0.5)] || 0;
          const p99v = latencies[Math.floor(latencies.length * 0.99)] || 0;
          const p999v = latencies[Math.floor(latencies.length * 0.999)] || 0;
          setP50(p50v);
          setP99(p99v);
          setP999(p999v);
          setLog((prev) => [
            ...prev.slice(-30),
            "═══════ TEST COMPLETE ═══════",
            `Total: ${totalCompletedRef.current.toLocaleString()} queries in ${elapsed.toFixed(1)}s`,
            `Throughput: ${Math.round(totalCompletedRef.current / elapsed).toLocaleString()} qps`,
            `p50: ${p50v}μs · p99: ${p99v}μs · p999: ${p999v}μs`,
          ]);
        }
      }, delay);
    };

    for (let i = 0; i < concurrency; i++) {
      setTimeout(() => sendQuery(), i * 5);
    }
    setLog((prev) => [...prev.slice(-30), `[INJECT] ${concurrency} concurrent streams launched`]);
  }, [running, concurrency, queryCount, saturated]);

  useEffect(() => {
    if (!running) return;
    intervalRef.current = setInterval(() => {
      const elapsed = (Date.now() - startTimeRef.current) / 1000;
      if (elapsed > 0) {
        setThroughput(Math.round(totalCompletedRef.current / elapsed));
      }
    }, 500);
    return () => {
      if (intervalRef.current) clearInterval(intervalRef.current);
    };
  }, [running]);

  const maxHistVal = Math.max(1, ...histogram);

  return (
    <div className="cyber-panel p-4 h-full flex flex-col gap-4 overflow-y-auto">
      <div className="text-[9px] font-mono font-bold text-[#f85149] tracking-wider uppercase pb-2 border-b border-border flex items-center gap-2">
        <Flame size={10} className="text-[#f85149]" />
        STRESS TEST
      </div>

      <div className="space-y-3">
        <div>
          <span className="text-[8px] font-mono text-text-muted block mb-1">CONCURRENCY</span>
          <div className="flex items-center gap-3">
            <input
              type="range" min="1" max="64" value={concurrency}
              onChange={(e) => setConcurrency(Number(e.target.value))}
              disabled={running}
              className="flex-1 accent-[#f85149]"
            />
            <span className="text-[10px] font-mono font-bold text-text w-8 text-right">{concurrency}</span>
          </div>
        </div>

        <div>
          <span className="text-[8px] font-mono text-text-muted block mb-1">QUERY COUNT</span>
          <div className="flex gap-2">
            {[100, 500, 1000, 5000, 10000].map((n) => (
              <button
                key={n}
                onClick={() => setQueryCount(n)}
                disabled={running}
                className={`px-2 py-1 text-[9px] font-mono rounded border transition-all ${
                  queryCount === n
                    ? "bg-[#f85149]/10 border-[#f85149]/30 text-[#f85149] font-bold"
                    : "bg-bg/30 border-border/30 text-text-muted hover:text-text"
                }`}
              >
                {n >= 1000 ? `${n / 1000}K` : n}
              </button>
            ))}
          </div>
        </div>
      </div>

      <button
        onClick={fire}
        disabled={running}
        className={`w-full flex items-center justify-center gap-2 py-3 rounded text-[11px] font-mono font-bold transition-all ${
          running
            ? "bg-[#f85149]/10 border border-[#f85149]/30 text-[#f85149]"
            : "bg-[#f85149]/10 border border-[#f85149]/30 text-[#f85149] hover:bg-[#f85149]/20"
        }`}
      >
        <Flame size={14} className={running ? "animate-pulse" : ""} />
        {running ? "BURNING..." : "FIRE"}
      </button>

      {running && (
        <div className="bg-[#f85149]/5 border border-[#f85149]/20 rounded p-2">
          <div className="flex justify-between text-[8px] font-mono">
            <span className="text-text-muted">PROGRESS</span>
            <span className="text-text font-bold">{completed}/{queryCount}</span>
          </div>
          <div className="w-full h-1.5 bg-bg rounded overflow-hidden mt-1">
            <div
              className="h-full bg-[#f85149] rounded transition-all duration-200"
              style={{ width: `${(completed / queryCount) * 100}%` }}
            />
          </div>
        </div>
      )}

      <div className="grid grid-cols-4 gap-2">
        <div className="bg-bg/50 border border-border/50 rounded p-2">
          <span className="text-[7px] font-mono text-text-muted block">DISPATCHED</span>
          <span className="text-lg font-mono font-bold text-text">{dispatched.toLocaleString()}</span>
        </div>
        <div className="bg-bg/50 border border-border/50 rounded p-2">
          <span className="text-[7px] font-mono text-text-muted block">COMPLETED</span>
          <span className="text-lg font-mono font-bold text-green">{completed.toLocaleString()}</span>
        </div>
        <div className="bg-bg/50 border border-border/50 rounded p-2">
          <span className="text-[7px] font-mono text-text-muted block">THROUGHPUT</span>
          <span className="text-lg font-mono font-bold text-blue">{throughput.toLocaleString()} qps</span>
        </div>
        <div className={`bg-bg/50 border rounded p-2 ${saturated ? "border-[#f85149]/50" : "border-border/50"}`}>
          <span className="text-[7px] font-mono text-text-muted block">QUEUE</span>
          <span className={`text-lg font-mono font-bold ${saturated ? "text-[#f85149]" : "text-green"}`}>
            {queueDepth}/{maxQueueDepth}
          </span>
        </div>
      </div>

      <div>
        <span className="text-[7px] font-mono text-text-muted block mb-2">LATENCY PERCENTILES</span>
        <div className="grid grid-cols-3 gap-3">
          <div className="bg-bg/50 border border-border/50 rounded p-2 text-center">
            <span className="text-[7px] font-mono text-text-muted block">p50</span>
            <span className="text-[10px] font-mono font-bold" style={{ color: latencyColor(p50) }}>
              {p50 > 0 ? `${(p50 / 1000).toFixed(1)}ms` : "—"}
            </span>
          </div>
          <div className="bg-bg/50 border border-border/50 rounded p-2 text-center">
            <span className="text-[7px] font-mono text-text-muted block">p99</span>
            <span className="text-[10px] font-mono font-bold" style={{ color: latencyColor(p99) }}>
              {p99 > 0 ? `${(p99 / 1000).toFixed(1)}ms` : "—"}
            </span>
          </div>
          <div className="bg-bg/50 border border-border/50 rounded p-2 text-center">
            <span className="text-[7px] font-mono text-text-muted block">p999</span>
            <span className="text-[10px] font-mono font-bold" style={{ color: latencyColor(p999) }}>
              {p999 > 0 ? `${(p999 / 1000).toFixed(1)}ms` : "—"}
            </span>
          </div>
        </div>
      </div>

      {histogram.some((v) => v > 0) && (
        <div>
          <span className="text-[7px] font-mono text-text-muted block mb-2">LATENCY DISTRIBUTION</span>
          <div className="space-y-1">
            {histogram.map((v, i) => (
              <div key={i} className="flex items-center gap-2">
                <span className="text-[7px] font-mono text-text-muted w-16 shrink-0">{buckets[i]}</span>
                <HistogramBar value={v} max={maxHistVal} color={i < 3 ? "#3fb950" : i < 5 ? "#d2991d" : "#f85149"} />
              </div>
            ))}
          </div>
        </div>
      )}

      <div className="flex-1" />

      <div className="space-y-1">
        <div className="text-[8px] font-mono text-text-soft font-bold uppercase">EVENT LOG</div>
        <div className="bg-bg border border-border/50 rounded p-2 text-[8px] font-mono text-text-soft leading-relaxed h-[100px] overflow-y-auto">
          {log.map((l, i) => <div key={i}>{l}</div>)}
        </div>
      </div>
    </div>
  );
}
