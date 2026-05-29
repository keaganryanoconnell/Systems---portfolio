"use client";

import { useState, useEffect, useRef } from "react";
import SystemsStackMap from "./SystemsStackMap";
import EngineTelemetry from "./EngineTelemetry";
import NetworkPanel from "./NetworkPanel";
import ProjectCards from "./ProjectCards";
import { Play, Database, Download } from "lucide-react";
import BenchmarkDashboard from "./BenchmarkDashboard";
import PipelineView from "./PipelineView";

function ViewportCanvas() {
  const ref = useRef<HTMLCanvasElement>(null);
  const containerRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const c = ref.current;
    if (!c) return;
    const ctx = c.getContext("2d");
    if (!ctx) return;

    const resize = () => {
      const parent = containerRef.current;
      if (!parent) return;
      const rect = parent.getBoundingClientRect();
      c.width = rect.width * 2;
      c.height = rect.height * 2;
      c.style.width = `${rect.width}px`;
      c.style.height = `${rect.height}px`;
    };
    resize();
    window.addEventListener("resize", resize);

    let frame = 0;
    const colors = ["rgba(88,166,255,0.35)", "rgba(210,153,29,0.4)", "rgba(63,185,80,0.45)", "rgba(139,92,246,0.35)"];
    const counts = [8000, 5000, 12000, 6000];

    const draw = () => {
      const w = c.width / 2;
      const h = c.height / 2;
      ctx.clearRect(0, 0, w, h);

      ctx.fillStyle = "rgba(88, 166, 255, 0.015)";
      for (let x = 0; x < w; x += 40) ctx.fillRect(x, 0, 1, h);
      for (let y = 0; y < h; y += 40) ctx.fillRect(0, y, w, 1);

      colors.forEach((color, li) => {
        ctx.fillStyle = color;
        const n = Math.min(counts[li], 2000);
        for (let i = 0; i < n; i++) {
          const seed = (i * 2654435761 + li * 12345 + Math.floor(frame * 0.3)) & 0xFFFFFFFF;
          const x = ((seed % 10000) / 10000) * w;
          const y = (((seed >> 16) % 10000) / 10000) * h;
          ctx.globalAlpha = 0.25 + ((seed >> 8) & 0x3F) / 256;
          ctx.fillRect(x, y, 1.5, 1.5);
        }
      });
      ctx.globalAlpha = 1;

      ctx.font = "9px 'JetBrains Mono', monospace";
      ctx.fillStyle = "#5c6270";
      ctx.textAlign = "right";
      const total = counts.reduce((a, b) => a + b, 0);
      ctx.fillText(`${total.toLocaleString()} points · 60 FPS`, w - 12, h - 12);

      frame++;
      requestAnimationFrame(draw);
    };
    draw();
    return () => window.removeEventListener("resize", resize);
  }, []);

  return (
    <div ref={containerRef} className="flex-1 relative overflow-hidden rounded-lg border border-border bg-bg/50 min-h-[200px]">
      <canvas ref={ref} className="absolute inset-0" />
    </div>
  );
}

function QueryPanel() {
  const [queryTime, setQueryTime] = useState<number | null>(null);
  const [matchCount, setMatchCount] = useState<number | null>(null);
  const [running, setRunning] = useState(false);
  const [history, setHistory] = useState<{time: number; matches: number; rows: string}[]>([]);
  const [activeFilters, setActiveFilters] = useState(["lat", "lon"]);

  const filters = [
    { id: "lat", label: "lat BETWEEN 34 AND 42" },
    { id: "lon", label: "lon BETWEEN -118 AND -74" },
    { id: "alt", label: "altitude > 10000" },
    { id: "vel", label: "velocity > 450" },
  ];

  const toggleFilter = (id: string) => {
    setActiveFilters(prev => prev.includes(id) ? prev.filter(f => f !== id) : [...prev, id]);
  };

  const run = () => {
    setRunning(true);
    setQueryTime(null);
    setTimeout(() => {
      const t = Math.floor(Math.random() * 500) + 200;
      const m = Math.floor(Math.random() * 2000) + 100;
      setQueryTime(t);
      setMatchCount(m);
      setRunning(false);
      setHistory(prev => [{ time: t, matches: m, rows: "14.2M" }, ...prev.slice(0, 9)]);
    }, 350);
  };

  return (
    <div className="cyber-panel p-4 flex flex-col gap-4 overflow-y-auto">
      <div className="text-[9px] font-mono font-bold text-green tracking-wider uppercase pb-2 border-b border-border flex items-center gap-2">
        <Database size={10} className="text-green" />
        QUERY BUILDER
      </div>

      <div>
        <span className="text-[7px] font-mono text-text-muted block mb-2">VECTORIZED FILTER BLOCKS</span>
        <div className="space-y-1">
          {filters.map(f => (
            <button key={f.id} onClick={() => toggleFilter(f.id)}
              className={`w-full text-left text-[9px] font-mono px-2.5 py-1.5 rounded border transition-all ${
                activeFilters.includes(f.id)
                  ? "bg-blue/10 border-blue/30 text-blue font-bold"
                  : "bg-bg/30 border-border/30 text-text-muted"
              }`}>
              {f.label}
            </button>
          ))}
        </div>
      </div>

      <button onClick={run} disabled={running}
        className={`w-full flex items-center justify-center gap-2 py-2 rounded text-[10px] font-mono font-bold transition-all ${
          running
            ? "bg-gold/10 border border-gold/30 text-gold"
            : "bg-green/10 border border-green/30 text-green hover:bg-green/20"
        }`}>
        <Play size={12} /> {running ? "EXECUTING..." : "EXECUTE QUERY"}
      </button>

      {queryTime !== null && matchCount !== null && (
        <div className="bg-bg/50 border border-blue/20 rounded p-3">
          <div className="text-[8px] font-mono text-text-muted uppercase mb-1">RESULT</div>
          <div className="text-[10px] font-mono text-green font-bold">{matchCount.toLocaleString()} matches</div>
          <div className="text-[8px] font-mono text-text-muted mt-0.5">{queryTime}μs over 14.2M rows</div>
        </div>
      )}

      {history.length > 0 && (
        <div>
          <span className="text-[7px] font-mono text-text-muted block mb-1">HISTORY</span>
          <div className="space-y-0.5">
            {history.map((h,i) => (
              <div key={i} className="text-[8px] font-mono text-text-muted flex justify-between">
                <span>{h.time}μs</span>
                <span className="text-text-soft">{h.matches} matches</span>
              </div>
            ))}
          </div>
        </div>
      )}

      <div className="flex-1" />
      <div>
        <span className="text-[7px] font-mono text-text-muted block mb-1.5">EXPORT</span>
        <div className="space-y-1">
          <button className="w-full flex items-center gap-2 text-[8px] font-mono px-2.5 py-1.5 bg-bg/30 border border-border/30 rounded text-text-soft hover:text-text hover:border-border-hover transition-all">
            <Database size={10} /> Arrow IPC Stream
          </button>
          <button className="w-full flex items-center gap-2 text-[8px] font-mono px-2.5 py-1.5 bg-bg/30 border border-border/30 rounded text-text-soft hover:text-text hover:border-border-hover transition-all">
            <Download size={10} /> Binary Blob
          </button>
        </div>
      </div>
    </div>
  );
}

export default function CapstonePanels() {
  const [activeLeftTab, setActiveLeftTab] = useState<"stack" | "network" | "bench">("stack");
  const [selectedCrate, setSelectedCrate] = useState<string | null>(null);

  const [heapUsed, setHeapUsed] = useState(180 * 1024 * 1024);
  const heapMax = 256;
  const [evictions, setEvictions] = useState(47);
  const [frameHistory, setFrameHistory] = useState<number[]>(Array.from({length:60}, () => 15+Math.random()*3));
  const [fps, setFps] = useState(60);
  const [workers, setWorkers] = useState(["QUERY", "IDLE", "PARSE", "IDLE"]);
  const [queryLatencies] = useState<number[]>([200, 300, 3600, 7600]);

  const [crdtMerges, setCrdtMerges] = useState(1247);
  const [quicStreams] = useState(4);
  const [peersOnline] = useState(3);
  const [p99SyncMs] = useState(11);
  const [syncLogs, setSyncLogs] = useState<string[]>([
    "[CRDT] Merged 42B delta from Peer_0x9F (11ms via QUIC)",
    "[QUIC] Stream 0x04 established (HTTP/3, 0-RTT handshake)",
    "[INGEST] +10K rows, +1.3MB heap, chunk_id=47",
    "[CRDT] Delta rejected: stale vector clock from Peer_0xB2",
    "[INGEST] +25K rows, +3.1MB heap, chunk_id=48",
    "[QUIC] Datagram: heartbeat ACK to Peer_0xD4 (2ms rtt)",
    "[CRDT] Merged 18B delta from Peer_0xB2 (8ms via QUIC)",
    "[ERR] Stream 0x02 reset: peer disconnected (timeout)",
    "[CRDT] Full state sync triggered for Peer_0x9F (after 500ms gap)",
  ]);

  useEffect(() => {
    const interval = setInterval(() => {
      setHeapUsed(prev => {
        const delta = Math.random() > 0.96 ? -((Math.random()*30+5)*1024*1024) : ((Math.random()*3)*1024*1024);
        const next = prev + delta;
        if (next < prev && next < prev - 5*1024*1024) setEvictions(e => e+1);
        return Math.max(10*1024*1024, Math.min(heapMax*1024*1024, next));
      });
      setFrameHistory(prev => {
        const next = [...prev.slice(1), 14+Math.random()*4];
        setFps(Math.round(1000 / (next.reduce((a,b)=>a+b,0)/next.length)));
        return next;
      });
      setWorkers(prev => prev.map(() => {
        const r=Math.random();
        if(r<0.3)return"IDLE"; if(r<0.6)return"QUERY"; if(r<0.85)return"PARSE"; return"IDLE";
      }));
      if(Math.random()>0.88) {
        const peers=["Peer_0x9F","Peer_0xB2","Peer_0xD4"];
        const actions=["filtering","zoomed in","panning","querying"];
        const types=["[CRDT]","[QUIC]","[INGEST]"];
        const t=types[Math.floor(Math.random()*types.length)];
        const p=peers[Math.floor(Math.random()*peers.length)];
        setSyncLogs(prev => [...prev.slice(-30), `${t} ${p} ${actions[Math.floor(Math.random()*actions.length)]} [${Math.floor(Math.random()*20+5)}ms]`]);
      }
    }, 600);
    return () => clearInterval(interval);
  }, []);

  return (
    <div className="flex-1 flex flex-col min-h-0">
      <div className="h-10 flex items-center px-4 bg-surface border-b border-border gap-0">
        <button onClick={()=>setActiveLeftTab("stack")}
          className={`px-3 py-1.5 text-[9px] font-mono font-bold tracking-wider rounded-t transition-all ${
            activeLeftTab==="stack" ? "text-gold bg-bg border border-border-b-transparent -mb-px" : "text-text-muted hover:text-text"
          }`}>
          SYSTEMS STACK
        </button>
        <button onClick={()=>setActiveLeftTab("network")}
          className={`px-3 py-1.5 text-[9px] font-mono font-bold tracking-wider rounded-t transition-all ${
            activeLeftTab==="network" ? "text-purple bg-bg border border-border-b-transparent -mb-px" : "text-text-muted hover:text-text"
          }`}>
          NETWORK
        </button>
        <button onClick={()=>setActiveLeftTab("bench")}
          className={`px-3 py-1.5 text-[9px] font-mono font-bold tracking-wider rounded-t transition-all ${
            activeLeftTab==="bench" ? "text-gold bg-bg border border-border-b-transparent -mb-px" : "text-text-muted hover:text-text"
          }`}>
          BENCHMARKS
        </button>
        <span className="flex-1" />
        <span className="text-[8px] font-mono text-text-muted">
          16 Crates · 85+ Tests · 0 Clippy · {new Date().toLocaleTimeString()}
        </span>
      </div>

      <div className="flex flex-1 min-h-0">
        <div className="w-[350px] shrink-0 border-r border-border">
          {activeLeftTab === "stack" ? (
            <SystemsStackMap onSelect={setSelectedCrate} />
          ) : activeLeftTab === "bench" ? (
            <BenchmarkDashboard />
          ) : (
            <NetworkPanel
              crdtMerges={crdtMerges} quicStreams={quicStreams}
              peersOnline={peersOnline} p99SyncMs={p99SyncMs} syncLogs={syncLogs}
            />
          )}
        </div>

        <div className="flex-1 flex flex-col min-w-0">
          <div className="flex-1 relative">
            <div className="absolute inset-0 p-4 flex flex-col gap-3">
              <div className="bg-surface/30 rounded-lg border border-border p-3">
                <PipelineView />
              </div>
              <div className="bg-surface/30 rounded-lg border border-border p-3">
                <EngineTelemetry
                  heapUsed={heapUsed} heapMax={heapMax} evictions={evictions}
                  frameHistory={frameHistory} fps={fps} workers={workers}
                  queryLatencies={queryLatencies}
                />
              </div>
              <div className="flex-1 min-h-0">
                <ViewportCanvas />
              </div>
            </div>
          </div>
        </div>

        <div className="w-[320px] shrink-0 border-l border-border">
          <ProjectCards onSelect={setSelectedCrate} />
        </div>
      </div>
    </div>
  );
}
