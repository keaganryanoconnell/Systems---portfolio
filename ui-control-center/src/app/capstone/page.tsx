"use client";

import { useState, useEffect } from "react";
import CapstoneHeader from "../components/CapstoneHeader";
import CapstonePanels from "../components/CapstonePanels";
import ShaderTransition from "../components/ShaderTransition";
import { EngineWorkerProvider, useWorkerEngine } from "../components/EngineWorkerProvider";

function CapstoneContent() {
  const {
    mode, toggleMode, sharedBufferAvailable, liveConnecting,
    activeWorkers, workerStates, heapUsed, heapMax,
  } = useWorkerEngine();

  const [fps, setFps] = useState(60);
  const [peerCount] = useState(3);
  const [uptime] = useState("4h 23m");

  useEffect(() => {
    const interval = setInterval(() => {
      setFps(Math.floor(58 + Math.random() * 3));
    }, 1000);
    return () => clearInterval(interval);
  }, []);

  return (
    <ShaderTransition>
      <div className="h-screen w-screen flex flex-col bg-bg text-text overflow-hidden select-none">
        <CapstoneHeader
          fps={fps}
          heapUsed={heapUsed}
          heapMax={heapMax}
          workers={mode === "live" ? workerStates : workerStates}
          peerCount={peerCount}
          uptime={uptime}
          mode={mode}
          onToggleMode={toggleMode}
          liveConnecting={liveConnecting}
          sharedBufferAvailable={sharedBufferAvailable}
        />
        <CapstonePanels workerMode={mode} />
      </div>
    </ShaderTransition>
  );
}

export default function CapstonePage() {
  return (
    <EngineWorkerProvider>
      <CapstoneContent />
    </EngineWorkerProvider>
  );
}
