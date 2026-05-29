"use client";

import { useState, useEffect } from "react";
import { Radio, Wifi, Activity } from "lucide-react";

export default function NetworkPanel({
  crdtMerges, quicStreams, peersOnline, p99SyncMs, syncLogs
}: {
  crdtMerges: number; quicStreams: number; peersOnline: number; p99SyncMs: number;
  syncLogs: string[];
}) {
  const [activeSet, setActiveSet] = useState<string[]>(["viewport_x:42", "viewport_y:18", "filter_lat:34-42", "filter_lon:-118--74", "query_active:true"]);
  const [conflictCount, setConflictCount] = useState(3);
  const [deltaHistory, setDeltaHistory] = useState<number[]>([42, 18, 31, 56, 24, 38, 42, 19]);

  useEffect(() => {
    const interval = setInterval(() => {
      if (Math.random() > 0.85) {
        const actions = ["add", "remove"];
        const action = actions[Math.floor(Math.random() * actions.length)];
        const peer = Math.floor(Math.random() * 3) + 1;
        if (action === "add") {
          const keys = ["zoom:1.5", "layer_grid:on", "layer_heatmap:off", "query_timeout:500ms"];
          const key = keys[Math.floor(Math.random() * keys.length)];
          if (!activeSet.includes(key)) {
            setActiveSet(prev => [...prev, key]);
            if (Math.random() > 0.7) setConflictCount(c => c + 1);
          }
        } else if (activeSet.length > 2) {
          setActiveSet(prev => prev.filter((_, i) => i !== Math.floor(Math.random() * prev.length)));
        }
      }
      setDeltaHistory(prev => {
        const next = [...prev.slice(1), Math.floor(Math.random() * 60) + 10];
        return next;
      });
    }, 1200);
    return () => clearInterval(interval);
  }, [activeSet]);

  return (
    <div className="cyber-panel p-4 h-full flex flex-col gap-4 overflow-y-auto">
      <div className="text-[9px] font-mono font-bold text-purple tracking-wider uppercase pb-2 border-b border-border flex items-center gap-2">
        <Radio size={10} className="text-purple animate-pulse-subtle" />
        NETWORK + CRDT
      </div>

      <div className="grid grid-cols-2 gap-3">
        <div className="bg-bg/50 border border-border/50 rounded p-2.5">
          <span className="text-[7px] font-mono text-text-muted block">CRDT MERGES</span>
          <div className="text-sm font-mono font-bold text-purple mt-0.5">{crdtMerges.toLocaleString()}</div>
        </div>
        <div className="bg-bg/50 border border-border/50 rounded p-2.5">
          <span className="text-[7px] font-mono text-text-muted block">QUIC STREAMS</span>
          <div className="text-sm font-mono font-bold text-blue mt-0.5">{quicStreams}</div>
        </div>
        <div className="bg-bg/50 border border-border/50 rounded p-2.5">
          <span className="text-[7px] font-mono text-text-muted block">PEERS ONLINE</span>
          <div className="text-sm font-mono font-bold text-green mt-0.5">{peersOnline}</div>
        </div>
        <div className="bg-bg/50 border border-border/50 rounded p-2.5">
          <span className="text-[7px] font-mono text-text-muted block">CONFLICTS</span>
          <div className="text-sm font-mono font-bold text-red mt-0.5">{conflictCount}</div>
        </div>
      </div>

      <div className="bg-bg/50 border border-purple/20 rounded p-2.5">
        <span className="text-[7px] font-mono text-text-muted block mb-2">LWW-ELEMENT-SET (ACTIVE ENTRIES)</span>
        <div className="space-y-0.5 max-h-[100px] overflow-y-auto">
          {activeSet.map((entry, i) => (
            <div key={i} className="flex items-center gap-2 text-[8px] font-mono text-text-soft">
              <span className="w-1 h-1 rounded-full bg-purple" />
              <span className="text-text-muted">{entry.split(":")[0]}:</span>
              <span className="text-text font-bold">{entry.split(":")[1]}</span>
            </div>
          ))}
        </div>
      </div>

      <div className="bg-bg/50 border border-blue/20 rounded p-2.5">
        <span className="text-[7px] font-mono text-text-muted block mb-2">DELTA SIZE HISTORY (bytes)</span>
        <div className="flex items-end gap-1 h-16">
          {deltaHistory.map((v, i) => {
            const pct = (v / 64) * 100;
            return (
              <div key={i} className="flex-1 flex flex-col items-center">
                <div className="w-full rounded-t" style={{ height: `${pct}%`, background: pct > 50 ? "#8b5cf6" : "#58a6ff", opacity: 0.6 + (i / deltaHistory.length) * 0.4 }} />
              </div>
            );
          })}
        </div>
        <div className="flex justify-between mt-1 text-[7px] font-mono text-text-muted">
          <span>0B</span>
          <span>64B</span>
        </div>
      </div>

      <div className="bg-bg/50 border border-border/50 rounded p-2.5">
        <span className="text-[7px] font-mono text-text-muted block mb-2">PROTOCOL STACK</span>
        <div className="space-y-1 text-[8px] font-mono">
          <div className="flex items-center gap-2 text-purple"><span className="w-1 h-1 rounded-full bg-purple" />CRDT Engine (LWW-Element-Set) — 7 tests</div>
          <div className="flex items-center gap-2 text-blue"><span className="w-1 h-1 rounded-full bg-blue" />WebTransport / QUIC (planned)</div>
          <div className="flex items-center gap-2 text-green"><span className="w-1 h-1 rounded-full bg-green" />HTTP/3 · 0-RTT Handshake (planned)</div>
          <div className="flex items-center gap-2 text-gold"><span className="w-1 h-1 rounded-full bg-gold" />Delta-Sync — byte-level diffs</div>
          <div className="flex items-center gap-2 text-green"><span className="w-1 h-1 rounded-full bg-green" />Merge — idempotent, convergent</div>
        </div>
      </div>

      <div className="flex-1 flex flex-col min-h-0">
        <span className="text-[7px] font-mono text-text-muted block mb-1">SYNC TICKER</span>
        <div className="flex-1 bg-bg/50 border border-border/50 rounded p-2 overflow-y-auto text-[8px] font-mono text-text-soft leading-relaxed">
          {syncLogs.map((l,i) => (
            <div key={i}>
              {l.startsWith("[CRDT]") ? <span className="text-purple">{l}</span> :
               l.startsWith("[QUIC]") ? <span className="text-blue">{l}</span> :
               l.startsWith("[INGEST]") ? <span className="text-green">{l}</span> :
               l.startsWith("[ERR]") ? <span className="text-red">{l}</span> :
               <span>{l}</span>}
            </div>
          ))}
        </div>
      </div>
    </div>
  );
}
