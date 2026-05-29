"use client";

import { useState, useEffect } from "react";
import { Activity, ArrowRight } from "lucide-react";

const STAGES = [
  { label: "INGEST", crates: ["api-gateway","telemetry-aggregator"], color: "#3fb950" },
  { label: "PARSE", crates: ["common-protocol","columnar-engine"], color: "#58a6ff" },
  { label: "STORE", crates: ["lsm-engine","raft-kv","log-broker"], color: "#8b5cf6" },
  { label: "QUERY", crates: ["sql-engine","columnar-engine","compute-orchestrator"], color: "#d2991d" },
  { label: "SYNC", crates: ["crdt-engine","sensor-fusion-buffer"], color: "#f85149" },
  { label: "RENDER", crates: ["columnar-engine","ui-control-center"], color: "#3fb950" },
];

export default function PipelineView() {
  const [activeStage, setActiveStage] = useState(0);
  const [throughput, setThroughput] = useState(0);

  useEffect(() => {
    const interval = setInterval(() => {
      setActiveStage(prev => (prev + 1) % STAGES.length);
      setThroughput(Math.floor(Math.random() * 50000) + 80000);
    }, 2000);
    return () => clearInterval(interval);
  }, []);

  return (
    <div className="cyber-panel p-4 flex flex-col gap-3">
      <div className="text-[9px] font-mono font-bold text-gold tracking-wider uppercase pb-2 border-b border-border flex items-center gap-2">
        <Activity size={10} className="text-green animate-pulse-subtle" />
        DATA PIPELINE
      </div>

      <div className="flex items-center gap-2">
        {STAGES.map((stage, i) => {
          const isActive = i === activeStage;
          return (
            <div key={stage.label} className="flex items-center gap-2">
              <button
                className={`px-3 py-2 rounded border text-center transition-all ${
                  isActive
                    ? "bg-bg border-border-hover shadow-sm"
                    : "bg-surface/30 border-border/30 opacity-60"
                }`}
                style={{ minWidth: 70 }}
              >
                <div className="text-[8px] font-mono font-bold tracking-wider" style={{ color: stage.color }}>
                  {stage.label}
                </div>
                <div className="text-[7px] font-mono text-text-muted mt-0.5">
                  {stage.crates.length} crates
                </div>
                {isActive && (
                  <div className="mt-1 text-[7px] font-mono" style={{ color: stage.color }}>
                    {throughput.toLocaleString()} ops/s
                  </div>
                )}
              </button>
              {i < STAGES.length - 1 && (
                <div className="text-text-muted">
                  <ArrowRight size={10} />
                </div>
              )}
            </div>
          );
        })}
      </div>

      <div className="text-[7px] font-mono text-text-muted flex items-center gap-2">
        <span className="w-1 h-1 rounded-full bg-green animate-pulse-subtle" />
        Active stage: <span className="text-text font-bold">{STAGES[activeStage].label}</span>
        <span className="text-text-muted">· Crates:</span>
        {STAGES[activeStage].crates.map(c => (
          <span key={c} className="bg-surface/50 px-1 rounded text-text-soft">{c}</span>
        ))}
      </div>
    </div>
  );
}

