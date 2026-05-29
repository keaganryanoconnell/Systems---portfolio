"use client";

import { Server } from "lucide-react";

const SERVICES = [
  { name: "api-gateway", port: "443:443", status: "healthy", image: "rust:1.85-alpine", desc: "HTTP/TLS entry point, axum 8 routes" },
  { name: "sql-engine", port: "—", status: "healthy", image: "rust:1.85-alpine", desc: "SQL parser, planner, executor" },
  { name: "raft-kv-0", port: "9000:9000", status: "healthy", image: "rust:1.85-alpine", desc: "Raft leader (or follower)" },
  { name: "raft-kv-1", port: "9001:9001", status: "healthy", image: "rust:1.85-alpine", desc: "Raft follower" },
  { name: "raft-kv-2", port: "9002:9002", status: "healthy", image: "rust:1.85-alpine", desc: "Raft follower" },
  { name: "lsm-engine", port: "—", status: "healthy", image: "rust:1.85-alpine", desc: "LSM storage (MemTable + SSTable)" },
  { name: "log-broker", port: "9092:9092", status: "healthy", image: "rust:1.85-alpine", desc: "Pub-sub segmented log broker" },
  { name: "compute-orchestrator", port: "9100:9100", status: "healthy", image: "rust:1.85-alpine", desc: "Actor system + SWIM gossip" },
  { name: "container-engine", port: "—", status: "warning", image: "rust:1.85-alpine", desc: "Linux container runtime (privileged)" },
  { name: "admin-tools", port: "—", status: "healthy", image: "rust:1.85-alpine", desc: "TUI dashboard (depends on api-gateway)" },
];

const STATUS_COLORS: Record<string, string> = { healthy: "#3fb950", warning: "#d2991d", error: "#f85149" };

export default function DeployPanel() {
  return (
    <div className="cyber-panel p-4 h-full flex flex-col gap-4 overflow-y-auto">
      <div className="text-[9px] font-mono font-bold text-gold tracking-wider uppercase pb-2 border-b border-border flex items-center gap-2">
        <Server size={10} className="text-gold" />
        DEPLOY TOPOLOGY
      </div>

      <div className="text-[7px] font-mono text-text-muted">
        docker compose up -d · 10 services · 3-node Raft cluster
      </div>

      <div className="space-y-1.5">
        {SERVICES.map((s, i) => (
          <div key={s.name} className={`bg-bg/50 border rounded p-2.5 transition-all ${
            s.status === "healthy" ? "border-border/50" : "border-gold/50"
          }`}>
            <div className="flex items-center justify-between mb-1">
              <div className="flex items-center gap-2">
                <span className="w-1.5 h-1.5 rounded-full" style={{ background: STATUS_COLORS[s.status] || "#5c6270" }} />
                <span className="text-[9px] font-mono font-bold text-text">{s.name}</span>
              </div>
              <span className="text-[7px] font-mono text-text-muted">{s.port}</span>
            </div>
            <div className="flex justify-between text-[7px] font-mono text-text-soft">
              <span>{s.desc}</span>
              <span className="text-text-muted">{s.image}</span>
            </div>
            {i < SERVICES.length - 1 && (
              <div className="ml-1.5 mt-1.5 mb-0.5 border-l border-border/30 h-2" />
            )}
          </div>
        ))}
      </div>

      <div className="mt-auto pt-3 border-t border-border">
        <div className="grid grid-cols-2 gap-2 text-[8px] font-mono">
          <div className="bg-bg/50 rounded p-2 text-center">
            <div className="text-text-muted">SERVICES</div>
            <div className="text-text font-bold text-sm">{SERVICES.length}</div>
          </div>
          <div className="bg-bg/50 rounded p-2 text-center">
            <div className="text-text-muted">HEALTHY</div>
            <div className="text-green font-bold text-sm">{SERVICES.filter(s => s.status === "healthy").length}</div>
          </div>
          <div className="bg-bg/50 rounded p-2 text-center">
            <div className="text-text-muted">STORAGE</div>
            <div className="text-gold font-bold text-sm">broker_data</div>
          </div>
          <div className="bg-bg/50 rounded p-2 text-center">
            <div className="text-text-muted">CMD</div>
            <div className="text-blue font-bold text-sm">compose up</div>
          </div>
        </div>
      </div>
    </div>
  );
}
