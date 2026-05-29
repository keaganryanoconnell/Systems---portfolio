"use client";

import { useState, useEffect, useRef } from "react";
import { Play, Square, ChevronDown, ChevronUp, Database, Download } from "lucide-react";

function DiagnosticsPanel({ heapUsed, heapMax, evictions, frameHistory, fps, workers }: {
  heapUsed: number; heapMax: number; evictions: number;
  frameHistory: number[]; fps: number; workers: string[];
}) {
  const heapPct = Math.min(100, (heapUsed / heapMax) * 100);

  return (
    <div className="cyber-panel h-full p-4 flex flex-col gap-4 overflow-y-auto">
      <div className="text-[9px] font-mono font-bold text-gold tracking-wider uppercase pb-2 border-b border-border">
        DIAGNOSTICS
      </div>

      <div>
        <div className="flex justify-between text-[8px] font-mono text-text-muted mb-1">
          <span>WASM LINEAR HEAP</span>
          <span>{(heapUsed/1024/1024).toFixed(1)}MB / {heapMax}MB</span>
        </div>
        <div className="h-2 bg-bg rounded overflow-hidden">
          <div className="h-full bg-gold rounded transition-all duration-700" style={{ width: `${heapPct}%` }} />
        </div>
        <div className="text-[8px] font-mono text-text-muted mt-1">
          LRU Evictions: <span className="text-red font-bold">{evictions}</span>
        </div>
      </div>

      <div>
        <div className="flex justify-between text-[8px] font-mono text-text-muted mb-1">
          <span>FRAME PIPELINE (μs)</span>
          <span className={fps >= 59 ? "text-green" : "text-red"}>{fps} FPS</span>
        </div>
        <FrameSparkline data={frameHistory} />
        <div className="text-[8px] font-mono text-text-muted mt-1">
          Target: 16.6ms <span className="text-text-soft">· Drops: 0</span>
        </div>
      </div>

      <div>
        <div className="text-[8px] font-mono text-text-muted mb-2">
          WORKER POOL
        </div>
        <div className="grid grid-cols-4 gap-1.5">
          {workers.map((state, i) => (
            <div key={i} className={`p-2 rounded text-center border ${
              state === "IDLE" ? "bg-bg/30 border-border/30" :
              state.includes("QUERY") ? "bg-green/10 border-green/30" :
              "bg-gold/10 border-gold/30"
            }`}>
              <div className="text-[9px] font-mono font-bold text-text">W{i+1}</div>
              <div className={`text-[7px] font-mono font-bold ${
                state === "IDLE" ? "text-text-muted" :
                state.includes("QUERY") ? "text-green" : "text-gold"
              }`}>{state}</div>
            </div>
          ))}
        </div>
      </div>

      <div className="text-[8px] font-mono text-text-muted flex items-center gap-1">
        <span className="w-1.5 h-1.5 rounded-full bg-green" />
        Engine: <span className="text-text font-bold">WebAssembly (wasm32)</span>
      </div>
      <div className="text-[8px] font-mono text-text-muted flex items-center gap-1">
        <span className="w-1.5 h-1.5 rounded-full bg-blue" />
        Runtime: <span className="text-text font-bold">tokio + axum + io_uring</span>
      </div>
      <div className="text-[8px] font-mono text-text-muted flex items-center gap-1">
        <span className="w-1.5 h-1.5 rounded-full bg-purple" />
        Network: <span className="text-text font-bold">WebTransport / QUIC</span>
      </div>
    </div>
  );
}

function FrameSparkline({ data }: { data: number[] }) {
  const ref = useRef<HTMLCanvasElement>(null);

  useEffect(() => {
    const c = ref.current;
    if (!c) return;
    const ctx = c.getContext("2d");
    if (!ctx) return;
    const w = c.width;
    const h = c.height;
    ctx.clearRect(0, 0, w, h);

    if (data.length < 2) return;

    const max = Math.max(...data, 20);
    const stepX = w / (data.length - 1);

    ctx.beginPath();
    ctx.strokeStyle = "#3fb950";
    ctx.lineWidth = 1;
    data.forEach((v, i) => {
      const x = i * stepX;
      const y = h - (v / max) * (h - 2);
      i === 0 ? ctx.moveTo(x, y) : ctx.lineTo(x, y);
    });
    ctx.stroke();

    ctx.strokeStyle = "rgba(248, 81, 73, 0.3)";
    ctx.setLineDash([2, 4]);
    ctx.beginPath();
    const y16 = h - (16.6 / max) * (h - 2);
    ctx.moveTo(0, y16);
    ctx.lineTo(w, y16);
    ctx.stroke();
    ctx.setLineDash([]);
  }, [data]);

  return <canvas ref={ref} width={200} height={40} className="w-full" />;
}

function ViewportPanel({ layers, layerToggles, onToggleLayer }: {
  layers: { id: string; name: string; color: string; points: number }[];
  layerToggles: Record<string, boolean>;
  onToggleLayer: (id: string) => void;
}) {
  const ref = useRef<HTMLCanvasElement>(null);
  const containerRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const c = ref.current;
    const container = containerRef.current;
    if (!c || !container) return;
    const ctx = c.getContext("2d");
    if (!ctx) return;

    const rect = container.getBoundingClientRect();
    c.width = rect.width * 2;
    c.height = rect.height * 2;
    c.style.width = `${rect.width}px`;
    c.style.height = `${rect.height}px`;
    ctx.scale(2, 2);

    const w = rect.width;
    const h = rect.height;

    let frame = 0;
    const draw = () => {
      ctx.clearRect(0, 0, w, h);

      ctx.fillStyle = "rgba(88, 166, 255, 0.02)";
      for (let x = 0; x < w; x += 40) {
        ctx.fillRect(x, 0, 1, h);
      }
      for (let y = 0; y < h; y += 40) {
        ctx.fillRect(0, y, w, 1);
      }

      layers.forEach((layer, li) => {
        if (!layerToggles[layer.id]) return;
        const count = Math.min(layer.points, 2000);
        ctx.fillStyle = layer.color;
        for (let i = 0; i < count; i++) {
          const seed = (i * 2654435761 + li * 12345 + frame) & 0xFFFFFFFF;
          const x = ((seed % 10000) / 10000) * w;
          const y = (((seed >> 16) % 10000) / 10000) * h;
          ctx.globalAlpha = 0.3;
          ctx.fillRect(x, y, 1.5, 1.5);
        }
      });

      ctx.globalAlpha = 1;

      ctx.font = "9px 'JetBrains Mono', monospace";
      ctx.fillStyle = "#5c6270";
      ctx.textAlign = "right";
      ctx.fillText(`${layers.reduce((a, l) => layerToggles[l.id] ? a + l.points : a, 0).toLocaleString()} points · 60 FPS`, w - 12, h - 12);

      frame++;
      requestAnimationFrame(draw);
    };
    draw();
  }, [layers, layerToggles]);

  return (
    <div ref={containerRef} className="flex-1 relative overflow-hidden rounded-lg border border-border bg-bg/50">
      <canvas ref={ref} className="absolute inset-0" />
    </div>
  );
}

function LayerCompositor({ layers, layerToggles, onToggleLayer, lod, onLodChange }: {
  layers: { id: string; name: string; color: string; points: number }[];
  layerToggles: Record<string, boolean>;
  onToggleLayer: (id: string) => void;
  lod: number;
  onLodChange: (v: number) => void;
}) {
  return (
    <div className="flex items-center gap-4 px-3 py-2 bg-surface border-b border-border">
      <span className="text-[8px] font-mono text-text-muted uppercase">LAYERS:</span>
      {layers.map(l => (
        <button
          key={l.id}
          onClick={() => onToggleLayer(l.id)}
          className={`flex items-center gap-1.5 text-[9px] font-mono px-2 py-0.5 rounded border transition-all ${
            layerToggles[l.id]
              ? "bg-bg border-border-hover text-text"
              : "border-transparent text-text-muted hover:text-text-soft"
          }`}
        >
          <span className="w-2 h-2 rounded-full" style={{ background: l.color }} />
          {l.name}
        </button>
      ))}
      <span className="text-[8px] font-mono text-text-muted mx-2">|</span>
      <span className="text-[8px] font-mono text-text-muted">LoD:</span>
      <input
        type="range" min={1} max={10} value={lod} onChange={e => onLodChange(parseInt(e.target.value))}
        className="w-20 h-1 accent-gold"
      />
      <span className="text-[8px] font-mono text-text font-bold">{lod}</span>
    </div>
  );
}

function CollaborationOverlay({ peers }: { peers: { id: string; action: string }[] }) {
  if (peers.length === 0) return null;
  return (
    <div className="absolute top-2 left-2 right-2 pointer-events-none">
      {peers.map((p, i) => (
        <div key={p.id} className="absolute" style={{
          left: `${20 + i * 25}%`, top: `${30 + i * 15}%`,
        }}>
          <div className="flex items-center gap-1.5 bg-surface/90 border border-purple/30 rounded px-2 py-0.5 text-[8px] font-mono">
            <span className="w-1.5 h-1.5 rounded-full bg-purple animate-pulse-subtle" />
            <span className="text-text">{p.id}</span>
            <span className="text-text-muted">({p.action})</span>
          </div>
        </div>
      ))}
    </div>
  );
}

function QueryBuilder({
  onRunQuery, queryTime, matchCount, scannedRows, filters, onToggleFilter,
}: {
  onRunQuery: () => void;
  queryTime: number | null;
  matchCount: number | null;
  scannedRows: string;
  filters: { id: string; label: string; active: boolean }[];
  onToggleFilter: (id: string) => void;
}) {
  return (
    <div className="cyber-panel h-full p-4 flex flex-col gap-4 overflow-y-auto">
      <div className="text-[9px] font-mono font-bold text-gold tracking-wider uppercase pb-2 border-b border-border">
        QUERY ENGINE
      </div>

      <div>
        <div className="text-[8px] font-mono text-text-muted uppercase mb-2">VECTORIZED FILTERS</div>
        <div className="space-y-1.5">
          {filters.map(f => (
            <button
              key={f.id}
              onClick={() => onToggleFilter(f.id)}
              className={`w-full text-left text-[10px] font-mono px-2.5 py-1.5 rounded border transition-all ${
                f.active
                  ? "bg-blue/10 border-blue/30 text-blue font-bold"
                  : "bg-bg/30 border-border/30 text-text-soft"
              }`}
            >
              {f.label}
            </button>
          ))}
        </div>
      </div>

      <button
        onClick={onRunQuery}
        className="w-full flex items-center justify-center gap-2 py-2 bg-green/10 border border-green/30 text-green text-[10px] font-mono font-bold rounded hover:bg-green/20 transition-all"
      >
        <Play size={12} /> EXECUTE QUERY
      </button>

      {queryTime !== null && matchCount !== null && (
        <div className="bg-bg/50 border border-border/50 rounded p-3">
          <div className="text-[8px] font-mono text-text-muted uppercase mb-1">RESULT</div>
          <div className="text-[10px] font-mono text-green font-bold">{matchCount.toLocaleString()} matches</div>
          <div className="text-[8px] font-mono text-text-muted mt-0.5">{queryTime}μs over {scannedRows} rows</div>
        </div>
      )}

      <div className="flex-1" />

      <div>
        <div className="text-[8px] font-mono text-text-muted uppercase mb-2">EXPORT</div>
        <div className="space-y-1.5">
          <button className="w-full flex items-center gap-2 text-[9px] font-mono px-2.5 py-1.5 bg-bg/30 border border-border/30 rounded text-text-soft hover:text-text hover:border-border-hover transition-all">
            <Database size={11} /> Arrow IPC Stream ↓
          </button>
          <button className="w-full flex items-center gap-2 text-[9px] font-mono px-2.5 py-1.5 bg-bg/30 border border-border/30 rounded text-text-soft hover:text-text hover:border-border-hover transition-all">
            <Download size={11} /> Binary Blob ↓
          </button>
        </div>
      </div>
    </div>
  );
}

function SyncTicker({ logs }: { logs: string[] }) {
  const ref = useRef<HTMLDivElement>(null);
  useEffect(() => { ref.current?.scrollTo({ top: ref.current.scrollHeight, behavior: "smooth" }); }, [logs]);

  return (
    <div className="p-3 bg-bg border-t border-border">
      <div className="text-[8px] font-mono text-text-muted uppercase mb-1.5">SYNC TICKER</div>
      <div ref={ref} className="h-14 overflow-y-auto text-[8px] font-mono text-text-soft leading-relaxed">
        {logs.map((l, i) => (
          <div key={i}>
            {l.startsWith("[CRDT]") ? <span className="text-purple">{l}</span> :
             l.startsWith("[QUIC]") ? <span className="text-blue">{l}</span> :
             l.startsWith("[INGEST]") ? <span className="text-green">{l}</span> :
             <span>{l}</span>}
          </div>
        ))}
      </div>
    </div>
  );
}

export default function CapstonePanels() {
  const [heapUsed, setHeapUsed] = useState(180 * 1024 * 1024);
  const heapMax = 256;
  const [evictions, setEvictions] = useState(3);
  const [frameHistory, setFrameHistory] = useState<number[]>(Array.from({ length: 60 }, () => 15.5 + Math.random() * 3));
  const [fps, setFps] = useState(60);
  const [workers, setWorkers] = useState(["IDLE", "IDLE", "IDLE", "IDLE"]);
  const [syncLogs, setSyncLogs] = useState<string[]>([
    "[CRDT] Merged 42B delta from Peer_0x9F (11ms via QUIC)",
    "[INGEST] +10K rows, +1.3MB heap",
    "[QUIC] Stream 0x04 established (HTTP/3, 0-RTT)",
  ]);
  const [peers] = useState([
    { id: "Peer_0x9F", action: "zoomed in" },
    { id: "Peer_0xB2", action: "filtering" },
  ]);
  const [queryTime, setQueryTime] = useState<number | null>(null);
  const [matchCount, setMatchCount] = useState<number | null>(null);
  const [queryRunning, setQueryRunning] = useState(false);
  const [lod, setLod] = useState(7);

  const [layers] = useState([
    { id: "grid", name: "Base Grid", color: "rgba(88,166,255,0.4)", points: 50000 },
    { id: "stream", name: "Stream", color: "rgba(210,153,29,0.5)", points: 25000 },
    { id: "hist", name: "Historical", color: "rgba(139,92,246,0.4)", points: 100000 },
    { id: "heat", name: "Heatmap", color: "rgba(248,81,73,0.5)", points: 75000 },
  ]);

  const [layerToggles, setLayerToggles] = useState<Record<string, boolean>>({
    grid: true, stream: true, hist: false, heat: true,
  });

  const [filters, setFilters] = useState([
    { id: "lat", label: "WHERE lat BETWEEN 34 AND 42", active: true },
    { id: "lon", label: "WHERE lon BETWEEN -118 AND -74", active: true },
    { id: "alt", label: "WHERE altitude > 10000", active: false },
    { id: "vel", label: "WHERE velocity > 450", active: false },
  ]);

  useEffect(() => {
    const interval = setInterval(() => {
      setHeapUsed(prev => {
        const delta = Math.random() > 0.97 ? -((Math.random() * 20 + 5) * 1024 * 1024) : ((Math.random() * 2) * 1024 * 1024);
        const next = prev + delta;
        if (next < prev && next < prev - 5 * 1024 * 1024) setEvictions(e => e + 1);
        return Math.max(10 * 1024 * 1024, Math.min(heapMax * 1024 * 1024, next));
      });

      setFrameHistory(prev => {
        const next = [...prev.slice(1), 14 + Math.random() * 4];
        const avg = next.reduce((a, b) => a + b) / next.length;
        setFps(Math.round(1000 / avg));
        return next;
      });

      setWorkers(prev => prev.map(() => {
        const r = Math.random();
        if (r < 0.3) return "IDLE";
        if (r < 0.6) return "QUERY";
        if (r < 0.85) return "PARSE";
        return "IDLE";
      }));

      if (Math.random() > 0.85) {
        const peers = ["Peer_0x9F", "Peer_0xB2", "Peer_0xD4"];
        const actions = ["filtering", "zoomed in", "panning", "querying"];
        const p = peers[Math.floor(Math.random() * peers.length)];
        const a = actions[Math.floor(Math.random() * actions.length)];
        setSyncLogs(prev => [...prev.slice(-30), `[CRDT] Merged delta from ${p} (${a}) [${Math.floor(Math.random()*20+5)}ms via QUIC]`]);
      }
    }, 500);
    return () => clearInterval(interval);
  }, []);

  const runQuery = () => {
    setQueryRunning(true);
    setQueryTime(null);
    setTimeout(() => {
      const t = Math.floor(Math.random() * 500) + 200;
      const m = Math.floor(Math.random() * 2000) + 100;
      setQueryTime(t);
      setMatchCount(m);
      setQueryRunning(false);
    }, 400);
  };

  const toggleFilter = (id: string) => {
    setFilters(prev => prev.map(f => f.id === id ? { ...f, active: !f.active } : f));
  };

  const toggleLayer = (id: string) => {
    setLayerToggles(prev => ({ ...prev, [id]: !prev[id] }));
  };

  return (
    <div className="flex-1 flex flex-col min-h-0">
      <div className="flex flex-1 min-h-0">
        <div className="w-[280px] shrink-0 border-r border-border">
          <DiagnosticsPanel
            heapUsed={heapUsed} heapMax={heapMax} evictions={evictions}
            frameHistory={frameHistory} fps={fps} workers={workers}
          />
        </div>

        <div className="flex-1 flex flex-col min-w-0 relative">
          <LayerCompositor
            layers={layers}
            layerToggles={layerToggles}
            onToggleLayer={toggleLayer}
            lod={lod}
            onLodChange={setLod}
          />
          <div className="flex-1 relative">
            <ViewportPanel layers={layers} layerToggles={layerToggles} onToggleLayer={toggleLayer} />
            <CollaborationOverlay peers={peers} />
          </div>
        </div>

        <div className="w-[260px] shrink-0 border-l border-border">
          <QueryBuilder
            onRunQuery={runQuery}
            queryTime={queryTime}
            matchCount={matchCount}
            scannedRows="14.2M"
            filters={filters}
            onToggleFilter={toggleFilter}
          />
        </div>
      </div>

      <SyncTicker logs={syncLogs} />
    </div>
  );
}
