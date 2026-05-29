"use client";

import { HardDrive } from "lucide-react";

const REGIONS = [
  { label: "CONTROL RING", start: 0, end: 24, color: "#d2991d", description: "Int32Array[6]: flags, task type, data offset/len, result offset/len" },
  { label: "INGEST BUFFER", start: 24, end: 24 + 67_108_864, color: "#3fb950", description: "1024 × 64KB slots for raw binary chunk ingestion" },
  { label: "WASM HEAP", start: 24 + 67_108_864, end: 24 + 67_108_864 + 33_554_432, color: "#58a6ff", description: "ColumnarEngine: chunk Vecs, BoundedBufferPool, LRU eviction" },
  { label: "RESULT BUFFER", start: 24 + 67_108_864 + 33_554_432, end: 24 + 67_108_864 + 33_554_432 + 33_554_432, color: "#8b5cf6", description: "Query result indices (u32 arrays), max 8M entries" },
];

const TOTAL_SIZE = 134_217_728; // 128MB

export default function MemoryMapPanel() {
  return (
    <div className="cyber-panel p-4 h-full flex flex-col gap-4 overflow-y-auto">
      <div className="text-[9px] font-mono font-bold text-gold tracking-wider uppercase pb-2 border-b border-border flex items-center gap-2">
        <HardDrive size={10} className="text-gold" />
        MEMORY MAP
      </div>

      <div className="text-[8px] font-mono text-text-muted">
        SharedArrayBuffer · 128MB · {TOTAL_SIZE.toLocaleString()} bytes
      </div>

      <div className="relative h-8 bg-bg rounded overflow-hidden flex">
        {REGIONS.map((r, i) => {
          const pct = ((r.end - r.start) / TOTAL_SIZE) * 100;
          return (
            <div
              key={r.label}
              className="h-full flex items-center justify-center text-[7px] font-mono font-bold text-white/80 transition-all hover:brightness-125"
              style={{ width: `${pct}%`, background: r.color, opacity: 0.7 + i * 0.1 }}
              title={`${r.label}: ${((r.end - r.start) / 1024 / 1024).toFixed(0)}MB`}
            >
              {pct > 8 ? r.label : ""}
            </div>
          );
        })}
      </div>

      <div className="space-y-2">
        {REGIONS.map((r, i) => (
          <div key={r.label} className="bg-bg/50 border border-border/50 rounded p-2.5">
            <div className="flex items-center justify-between mb-1">
              <div className="flex items-center gap-2">
                <span className="w-2 h-2 rounded-sm" style={{ background: r.color }} />
                <span className="text-[9px] font-mono font-bold text-text">{r.label}</span>
              </div>
              <span className="text-[8px] font-mono text-text-soft">
                {((r.end - r.start) / 1024 / 1024).toFixed(0)}MB
              </span>
            </div>
            <div className="flex justify-between text-[7px] font-mono text-text-muted">
              <span>0x{(r.start).toString(16).padStart(8, '0')}</span>
              <span>0x{(r.end).toString(16).padStart(8, '0')}</span>
            </div>
            <div className="text-[7px] font-mono text-text-soft mt-1">{r.description}</div>
          </div>
        ))}
      </div>

      <div className="mt-auto pt-3 border-t border-border">
        <div className="grid grid-cols-2 gap-2 text-[8px] font-mono">
          <div className="bg-bg/50 rounded p-2 text-center">
            <div className="text-text-muted">CACHE LINE</div>
            <div className="text-text font-bold text-sm">64B</div>
          </div>
          <div className="bg-bg/50 rounded p-2 text-center">
            <div className="text-text-muted">ALIGNMENT</div>
            <div className="text-text font-bold text-sm">64B</div>
          </div>
          <div className="bg-bg/50 rounded p-2 text-center">
            <div className="text-text-muted">PAGES</div>
            <div className="text-text font-bold text-sm">32K</div>
          </div>
          <div className="bg-bg/50 rounded p-2 text-center">
            <div className="text-text-muted">WORKERS</div>
            <div className="text-green font-bold text-sm">4</div>
          </div>
        </div>
      </div>
    </div>
  );
}
