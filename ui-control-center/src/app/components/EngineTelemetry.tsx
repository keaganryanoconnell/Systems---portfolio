"use client";

import { useEffect, useRef, useState } from "react";
import { Activity } from "lucide-react";

export default function EngineTelemetry({
  heapUsed, heapMax, evictions, frameHistory, fps, workers, queryLatencies,
}: {
  heapUsed: number; heapMax: number; evictions: number;
  frameHistory: number[]; fps: number; workers: string[];
  queryLatencies: number[];
}) {
  return (
    <div className="cyber-panel p-4 h-full flex flex-col gap-4 overflow-y-auto">
      <div className="text-[9px] font-mono font-bold text-gold tracking-wider uppercase pb-2 border-b border-border flex items-center gap-2">
        <Activity size={10} className="text-blue animate-pulse-subtle" />
        ENGINE TELEMETRY
      </div>

      <div className="grid grid-cols-2 gap-3">
        <HeapGauge heapUsed={heapUsed} heapMax={heapMax} evictions={evictions} />
        <FramePipeline frameHistory={frameHistory} fps={fps} />
      </div>

      <WorkerPool workers={workers} />
      <LatencyHistogram latencies={queryLatencies} />
      <SystemMetrics />
    </div>
  );
}

function HeapGauge({ heapUsed, heapMax, evictions }: { heapUsed: number; heapMax: number; evictions: number }) {
  const pct = Math.min(100, (heapUsed / heapMax) * 100);
  return (
    <div className="bg-bg/50 border border-border/50 rounded p-2.5">
      <span className="text-[7px] font-mono text-text-muted block">WASM LINEAR HEAP</span>
      <div className="text-sm font-mono font-bold text-text mt-0.5">
        {(heapUsed/1024/1024).toFixed(0)}<span className="text-text-soft">/{heapMax}MB</span>
      </div>
      <div className="h-2 bg-bg rounded overflow-hidden mt-1">
        <div className="h-full bg-gold rounded transition-all duration-700" style={{ width: `${pct}%` }} />
      </div>
      <div className="text-[7px] font-mono text-red mt-1">
        LRU Evictions: <span className="font-bold">{evictions}</span>
      </div>
    </div>
  );
}

function FramePipeline({ frameHistory, fps }: { frameHistory: number[]; fps: number }) {
  const ref = useRef<HTMLCanvasElement>(null);
  useEffect(() => {
    const c = ref.current; if (!c) return;
    const ctx = c.getContext("2d"); if (!ctx) return;
    const w=120,h=50;
    ctx.clearRect(0,0,w,h);
    if (frameHistory.length<2) return;
    const max=20;
    ctx.beginPath(); ctx.strokeStyle="#3fb950"; ctx.lineWidth=1;
    frameHistory.forEach((v,i)=>{
      const x=i/(frameHistory.length-1)*w;
      const y=h-(v/max)*(h-4);
      i===0?ctx.moveTo(x,y):ctx.lineTo(x,y);
    });
    ctx.stroke();
    ctx.strokeStyle="rgba(248,81,73,0.3)"; ctx.setLineDash([2,2]);
    ctx.beginPath(); const y16=h-(16.6/max)*(h-4); ctx.moveTo(0,y16); ctx.lineTo(w,y16); ctx.stroke();
    ctx.setLineDash([]);
  },[frameHistory]);
  return (
    <div className="bg-bg/50 border border-border/50 rounded p-2.5">
      <span className="text-[7px] font-mono text-text-muted block">FRAME PIPELINE (μs)</span>
      <div className="text-sm font-mono font-bold text-text mt-0.5">
        <span className={fps>=59?"text-green":"text-red"}>{fps} FPS</span>
        <span className="text-text-soft text-[9px] ml-1">· 0 drops</span>
      </div>
      <canvas ref={ref} width={120} height={50} className="w-full mt-1" />
    </div>
  );
}

function WorkerPool({ workers }: { workers: string[] }) {
  return (
    <div className="bg-bg/50 border border-border/50 rounded p-2.5">
      <span className="text-[7px] font-mono text-text-muted block mb-2">WORKER POOL</span>
      <div className="flex gap-2">
        {workers.map((s,i) => {
          const active = s !== "IDLE";
          const color = s.includes("QUERY") ? "#3fb950" : s.includes("PARSE") ? "#d2991d" : "#252a36";
          const pct = active ? (s.includes("QUERY") ? 85 : 45) : 5;
          return (
            <div key={i} className="flex-1 text-center">
              <div className="text-[8px] font-mono font-bold text-text">W{i+1}</div>
              <div className="h-8 bg-bg rounded overflow-hidden mt-0.5 relative">
                <div className="absolute bottom-0 w-full rounded transition-all duration-500" style={{height:`${pct}%`, background:color}} />
              </div>
              <div className="text-[7px] font-mono mt-0.5" style={{color}}>{s}</div>
            </div>
          );
        })}
      </div>
    </div>
  );
}

function LatencyHistogram({ latencies }: { latencies: number[] }) {
  return (
    <div className="bg-bg/50 border border-border/50 rounded p-2.5">
      <span className="text-[7px] font-mono text-text-muted block">QUERY LATENCY (ns)</span>
      <div className="text-[10px] font-mono text-text mt-1 space-y-0.5">
        {[
          {l:"p50",v:200},
          {l:"p90",v:300},
          {l:"p99",v:3600},
          {l:"max",v:7600},
        ].map(r=>(
          <div key={r.l} className="flex justify-between">
            <span className="text-text-muted">{r.l}</span>
            <span className="text-text font-bold">{r.v.toLocaleString()}ns</span>
          </div>
        ))}
      </div>
    </div>
  );
}

function SystemMetrics() {
  return (
    <div className="bg-bg/50 border border-border/50 rounded p-2.5">
      <span className="text-[7px] font-mono text-text-muted block">SYSTEM METRICS</span>
      <div className="grid grid-cols-2 gap-1.5 mt-1 text-[8px] font-mono">
        <div><span className="text-text-muted">Crates:</span> <span className="text-text font-bold">16</span></div>
        <div><span className="text-text-muted">Tests:</span> <span className="text-green font-bold">85+ PASS</span></div>
        <div><span className="text-text-muted">Clippy:</span> <span className="text-green font-bold">0 warnings</span></div>
        <div><span className="text-text-muted">Bench:</span> <span className="text-gold font-bold">Criterion</span></div>
        <div><span className="text-text-muted">Rust:</span> <span className="text-text font-bold">1.85</span></div>
        <div><span className="text-text-muted">Wasm:</span> <span className="text-text font-bold">wasm32</span></div>
      </div>
    </div>
  );
}
