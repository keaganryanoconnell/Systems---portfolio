"use client";

import { BarChart3 } from "lucide-react";

const BENCHMARKS = [
  { crate: "core-sys", bench: "SPSC Queue", metric: "50ns push/pop", p50: 50, p99: 80, color: "#58a6ff" },
  { crate: "lob-engine", bench: "LOB Matching", metric: "221ns/order", p50: 200, p99: 3600, color: "#3fb950" },
  { crate: "lsm-engine", bench: "LSM Writes", metric: "120K writes/s", p50: 8000, p99: 25000, color: "#d2991d" },
  { crate: "columnar-engine", bench: "Columnar Scan", metric: "200ns/65K rows", p50: 200, p99: 3600, color: "#58a6ff" },
  { crate: "sensor-fusion-buffer", bench: "MPMC Buffer", metric: "30K frames", p50: 150, p99: 1200, color: "#8b5cf6" },
  { crate: "telemetry-aggregator", bench: "Compression", metric: "3.1:1 ratio", p50: 500, p99: 8000, color: "#d2991d" },
  { crate: "log-broker", bench: "Log Broker", metric: "17 tests", p50: 300, p99: 5000, color: "#3fb950" },
  { crate: "compute-orchestrator", bench: "Actor System", metric: "7 tests", p50: 400, p99: 7000, color: "#8b5cf6" },
];

export default function BenchmarkDashboard() {
  const maxP99 = Math.max(...BENCHMARKS.map(b => b.p99));

  return (
    <div className="cyber-panel p-4 h-full flex flex-col gap-4 overflow-y-auto">
      <div className="text-[9px] font-mono font-bold text-gold tracking-wider uppercase pb-2 border-b border-border flex items-center gap-2">
        <BarChart3 size={10} className="text-gold" />
        BENCHMARK PROFILES
      </div>

      <div className="space-y-2">
        {BENCHMARKS.map(b => {
          const p50Pct = (b.p50 / maxP99) * 100;
          const p99Pct = (b.p99 / maxP99) * 100;

          return (
            <div key={b.bench} className="bg-bg/50 border border-border/50 rounded p-2.5">
              <div className="flex items-center justify-between mb-1.5">
                <div className="flex items-center gap-2">
                  <span className="w-1.5 h-1.5 rounded-full" style={{ background: b.color }} />
                  <span className="text-[9px] font-mono font-bold text-text">{b.bench}</span>
                  <span className="text-[7px] font-mono text-text-muted">[{b.crate}]</span>
                </div>
                <span className="text-[8px] font-mono text-text-soft">{b.metric}</span>
              </div>

              <div className="flex items-center gap-3">
                <div className="flex-1">
                  <div className="flex justify-between text-[7px] font-mono text-text-muted mb-0.5">
                    <span>p50</span>
                    <span>{b.p50}ns</span>
                  </div>
                  <div className="h-1.5 bg-bg rounded overflow-hidden">
                    <div className="h-full rounded transition-all duration-700" style={{ width: `${p50Pct}%`, background: b.color }} />
                  </div>
                  <div className="flex justify-between text-[7px] font-mono text-text-muted mt-0.5">
                    <span>p99</span>
                    <span>{b.p99}ns</span>
                  </div>
                  <div className="h-1.5 bg-bg rounded overflow-hidden mt-0.5">
                    <div className="h-full rounded transition-all duration-700" style={{ width: `${p99Pct}%`, background: b.color, opacity: 0.6 }} />
                  </div>
                </div>
              </div>
            </div>
          );
        })}
      </div>

      <div className="mt-auto pt-3 border-t border-border">
        <div className="grid grid-cols-2 gap-2 text-[8px] font-mono">
          <div className="bg-bg/50 rounded p-2 text-center">
            <div className="text-text-muted">TOTAL BENCHES</div>
            <div className="text-text font-bold text-sm">8</div>
          </div>
          <div className="bg-bg/50 rounded p-2 text-center">
            <div className="text-text-muted">CRITERION</div>
            <div className="text-green font-bold text-sm">PASS</div>
          </div>
          <div className="bg-bg/50 rounded p-2 text-center">
            <div className="text-text-muted">FASTEST P50</div>
            <div className="text-gold font-bold text-sm">50ns</div>
          </div>
          <div className="bg-bg/50 rounded p-2 text-center">
            <div className="text-text-muted">FASTEST P99</div>
            <div className="text-blue font-bold text-sm">80ns</div>
          </div>
        </div>
      </div>
    </div>
  );
}
