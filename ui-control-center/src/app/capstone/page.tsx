"use client";

import { useState, useEffect } from "react";
import CapstoneHeader from "../components/CapstoneHeader";
import CapstonePanels from "../components/CapstonePanels";

export default function CapstonePage() {
  const [fps, setFps] = useState(60);
  const [heapUsed, setHeapUsed] = useState(180 * 1024 * 1024);
  const [workers, setWorkers] = useState(["QUERY", "IDLE", "PARSE", "IDLE"]);
  const [uptime] = useState("4h 23m");

  useEffect(() => {
    const interval = setInterval(() => {
      setFps(Math.floor(58 + Math.random() * 3));
      setHeapUsed(prev => {
        const delta = Math.random() > 0.97 ? -((Math.random() * 20 + 5) * 1024 * 1024) : ((Math.random() * 2) * 1024 * 1024);
        return Math.max(10 * 1024 * 1024, Math.min(256 * 1024 * 1024, prev + delta));
      });
      setWorkers(prev => prev.map(() => {
        const r = Math.random();
        if (r < 0.3) return "IDLE";
        if (r < 0.6) return "QUERY";
        if (r < 0.85) return "PARSE";
        return "IDLE";
      }));
    }, 1000);
    return () => clearInterval(interval);
  }, []);

  return (
    <div className="h-screen w-screen flex flex-col bg-bg text-text overflow-hidden select-none">
      <CapstoneHeader
        fps={fps}
        heapUsed={heapUsed}
        heapMax={256}
        workers={workers}
        peerCount={3}
        uptime={uptime}
      />
      <CapstonePanels />
    </div>
  );
}
