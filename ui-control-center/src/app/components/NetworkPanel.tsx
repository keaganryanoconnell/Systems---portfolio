"use client";

import { Radio, Wifi } from "lucide-react";

export default function NetworkPanel({
  crdtMerges, quicStreams, peersOnline, p99SyncMs, syncLogs
}: {
  crdtMerges: number; quicStreams: number; peersOnline: number; p99SyncMs: number;
  syncLogs: string[];
}) {
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
          <span className="text-[7px] font-mono text-text-muted block">P99 STATE SYNC</span>
          <div className="text-sm font-mono font-bold text-text mt-0.5">{p99SyncMs}ms</div>
        </div>
      </div>

      <div className="bg-bg/50 border border-border/50 rounded p-2.5">
        <span className="text-[7px] font-mono text-text-muted block mb-2">PROTOCOL STACK</span>
        <div className="space-y-1 text-[8px] font-mono">
          <div className="flex items-center gap-2 text-purple"><span className="w-1 h-1 rounded-full bg-purple" />CRDT Engine (LWW-Element-Set)</div>
          <div className="flex items-center gap-2 text-blue"><span className="w-1 h-1 rounded-full bg-blue" />WebTransport / QUIC</div>
          <div className="flex items-center gap-2 text-green"><span className="w-1 h-1 rounded-full bg-green" />HTTP/3 · 0-RTT Handshake</div>
          <div className="flex items-center gap-2 text-gold"><span className="w-1 h-1 rounded-full bg-gold" />Delta-Sync (byte-level diffs)</div>
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
