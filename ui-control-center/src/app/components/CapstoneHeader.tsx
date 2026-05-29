"use client";

import { Activity, Cpu, HardDrive, Layers, Wifi, Zap, Radio } from "lucide-react";

export default function CapstoneHeader({
  fps, heapUsed, heapMax, workers, peerCount, uptime,
  mode, onToggleMode, liveConnecting, sharedBufferAvailable,
}: {
  fps: number; heapUsed: number; heapMax: number;
  workers: string[]; peerCount: number; uptime: string;
  mode: "sim" | "live";
  onToggleMode: () => void;
  liveConnecting: boolean;
  sharedBufferAvailable: boolean;
}) {
  const heapPct = Math.min(100, (heapUsed / heapMax) * 100);
  const activeWorkers = workers.filter(w => w !== "IDLE").length;

  return (
    <header className="h-14 flex items-center justify-between px-5 bg-surface border-b border-border shrink-0">
      <div className="flex items-center gap-4">
        <span className="text-gold font-mono text-sm font-bold">◈</span>
        <h1 className="text-xs font-mono font-black tracking-tight text-text">
          SPATIAL<span className="text-text-soft">_ANALYTICS_ENGINE</span>
        </h1>
        <span className="text-[9px] font-mono text-text-muted border border-border px-2 py-0.5 rounded">v0.1.0</span>
        <span className="text-[8px] font-mono text-text-muted hidden sm:inline">
          20 Crates · 85+ Tests · 0 Clippy
        </span>
      </div>

      <div className="flex items-center gap-5 text-[10px] font-mono">
        <button
          onClick={onToggleMode}
          disabled={!sharedBufferAvailable || liveConnecting}
          className={`flex items-center gap-1.5 px-2.5 py-1 rounded border text-[9px] font-bold transition-all ${
            mode === "live"
              ? "bg-green/10 border-green/30 text-green"
              : liveConnecting
              ? "bg-gold/10 border-gold/30 text-gold animate-pulse-subtle"
              : !sharedBufferAvailable
              ? "bg-surface border-border/30 text-text-muted opacity-50 cursor-not-allowed"
              : "bg-surface border-border/30 text-text-muted hover:border-green/30 hover:text-green"
          }`}
          title={
            liveConnecting ? "Connecting to workers..." :
            !sharedBufferAvailable ? "SharedArrayBuffer unavailable — requires cross-origin isolation headers" :
            mode === "live" ? "Click to return to SIM mode" :
            "Click to switch to LIVE worker mode"
          }
        >
          {liveConnecting ? (
            <Radio size={11} className="text-gold animate-pulse" />
          ) : (
            <Zap size={11} className={mode === "live" ? "text-green" : "text-text-muted"} />
          )}
          {liveConnecting ? "CONNECTING..." : mode === "live" ? "LIVE" : "SIM"}
        </button>

        <div className="flex items-center gap-1.5">
          <Activity size={12} className="text-green" />
          <span className="text-text-muted">FPS:</span>
          <span className={fps >= 59 ? "text-green font-bold" : "text-red font-bold"}>{fps}</span>
        </div>

        <div className="flex items-center gap-1.5">
          <HardDrive size={12} className="text-gold" />
          <span className="text-text-muted">HEAP:</span>
          <span className="text-text font-bold">{(heapUsed / 1024 / 1024).toFixed(0)}MB</span>
          <span className="text-text-muted">/ {(heapMax / 1024 / 1024).toFixed(0)}MB</span>
          <div className="w-12 h-1.5 bg-bg rounded overflow-hidden ml-1">
            <div className="h-full bg-gold rounded transition-all duration-500" style={{ width: `${heapPct}%` }} />
          </div>
        </div>

        <div className="flex items-center gap-1.5">
          <Cpu size={12} className="text-blue" />
          <span className="text-text-muted">WORKERS:</span>
          <span className="text-text font-bold">{activeWorkers}/{workers.length}</span>
          <div className="flex gap-0.5 ml-1">
            {workers.map((s, i) => (
              <span key={i} className={`w-1.5 h-1.5 rounded-full ${
                s === "IDLE" ? "bg-border" : s.includes("QUERY") ? "bg-green animate-pulse-subtle" : "bg-gold"
              }`} />
            ))}
          </div>
        </div>

        <div className="flex items-center gap-1.5 pl-3 border-l border-border">
          <Wifi size={12} className="text-purple" />
          <span className="text-text-muted">PEERS:</span>
          <span className="text-text font-bold">{peerCount}</span>
          <span className="text-text-muted">· {uptime}</span>
        </div>
      </div>
    </header>
  );
}
