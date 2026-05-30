"use client";

import { Music } from "lucide-react";

export default function SpotifyNowPlaying() {
  return (
    <div className="cyber-panel p-4 border-l-4 border-l-[#1DB954]">
      <div className="flex items-center gap-2 mb-3 pb-2 border-b border-border">
        <Music size={14} className="text-[#1DB954]" />
        <span className="text-[9px] font-mono font-bold text-[#1DB954] tracking-wider uppercase">
          Spotify
        </span>
      </div>
      <div className="space-y-3">
        <div className="flex items-center gap-3">
          <div className="w-10 h-10 rounded bg-[#1DB954]/20 flex items-center justify-center">
            <Music size={16} className="text-[#1DB954]" />
          </div>
          <div className="flex-1 min-w-0">
            <div className="text-[10px] font-mono font-bold text-text truncate">
              Not Playing
            </div>
            <div className="text-[9px] font-mono text-text-muted truncate">
              Spotify API offline
            </div>
          </div>
        </div>
        <div className="w-full h-1 bg-bg rounded overflow-hidden">
          <div className="h-full bg-[#1DB954]/30 rounded" style={{ width: "0%" }} />
        </div>
        <div className="text-[8px] font-mono text-text-muted text-right">
          Connect Spotify API for live now-playing
        </div>
      </div>
    </div>
  );
}
