"use client";

import { useState, useEffect, useCallback, useRef } from "react";
import { decodeNodeTelemetry, encodeNodeTelemetry, generateMockNodesTelemetry, type NodeTelemetry } from "./utils/tauri";
import ErrorBoundary from "./components/ErrorBoundary";
import NavBar from "./components/NavBar";
import Hero from "./components/Hero";
import ArchMap from "./components/ArchMap";
import ProjectWorkspace from "./components/ProjectWorkspace";
import TelemetryChart from "./components/TelemetryChart";
import Forum from "./components/Forum";
import DeepDives from "./components/DeepDives";
import { About, Footer } from "./components/AboutFooter";

const MAX_HISTORY = 45;

export default function Portfolio() {
  const [chaosMode, setChaosMode] = useState({
    partitionSplit: false,
    malformedFrames: false,
    crashNode2: false,
    fuzzerRunning: false,
  });
  const [nodes, setNodes] = useState<NodeTelemetry[]>([]);
  const [history, setHistory] = useState<{
    cpu: Record<number, number[]>;
    memory: Record<number, number[]>;
    fd: Record<number, number[]>;
  }>({ cpu: {}, memory: {}, fd: {} });
  const [overviewCpu, setOverviewCpu] = useState<number[]>([]);
  const [overviewIops, setOverviewIops] = useState<number[]>([]);
  const [overviewMemory, setOverviewMemory] = useState<number[]>([]);
  const telemetryRef = useRef<ReturnType<typeof setInterval> | null>(null);

  const telemetryTick = useCallback(() => {
    const mockData = generateMockNodesTelemetry(chaosMode);
    const binary = encodeNodeTelemetry(mockData);
    const decoded = decodeNodeTelemetry(binary);

    setNodes(decoded);

    setHistory((prev) => {
      const cpu = { ...prev.cpu };
      const memory = { ...prev.memory };
      const fd = { ...prev.fd };
      decoded.forEach((node) => {
        cpu[node.nodeId] = [...(cpu[node.nodeId] || []).slice(-(MAX_HISTORY - 1)), node.cpu];
        memory[node.nodeId] = [...(memory[node.nodeId] || []).slice(-(MAX_HISTORY - 1)), node.arenaMemoryAllocated];
        fd[node.nodeId] = [...(fd[node.nodeId] || []).slice(-(MAX_HISTORY - 1)), node.activeFdPool];
      });
      return { cpu, memory, fd };
    });

    setOverviewCpu((prev) => [
      ...prev.slice(-(MAX_HISTORY - 1)),
      decoded.reduce((s, n) => s + n.cpu, 0) / Math.max(1, decoded.length),
    ]);
    setOverviewIops((prev) => [
      ...prev.slice(-(MAX_HISTORY - 1)),
      decoded.reduce((s, n) => s + n.iops, 0),
    ]);
    setOverviewMemory((prev) => [
      ...prev.slice(-(MAX_HISTORY - 1)),
      decoded.reduce((s, n) => s + n.arenaMemoryAllocated, 0),
    ]);
  }, [chaosMode]);

  useEffect(() => {
    telemetryTick();
    telemetryRef.current = setInterval(telemetryTick, 500);
    return () => {
      if (telemetryRef.current) clearInterval(telemetryRef.current);
    };
  }, [telemetryTick]);

  const telemetryDemo = (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <div>
          <span className="text-[10px] font-mono font-bold text-gold tracking-widest uppercase block">
            REAL-TIME CLUSTER METRICS
          </span>
          <h3 className="text-xl font-extrabold text-text mt-0.5">Global Telemetry Stream</h3>
        </div>
        <div className="flex items-center gap-2 text-[10px] font-mono text-text-soft">
          <span className="w-2 h-2 rounded-full bg-green animate-pulse" />
          <span>POLLING ACTIVE (2Hz)</span>
        </div>
      </div>
      <div className="cyber-panel h-[280px] p-0 overflow-hidden relative">
        <div className="absolute top-0 left-0 right-0 h-px bg-gradient-to-r from-transparent via-gold/30 to-transparent" />
        <TelemetryChart
          cpuHistory={overviewCpu}
          iopsHistory={overviewIops}
          memoryHistory={overviewMemory}
        />
      </div>
    </div>
  );

  return (
    <div className="min-h-screen bg-bg text-text">
      <NavBar />
      <ErrorBoundary>
        <Hero />

        <section id="metrics" className="section py-12">
          {telemetryDemo}
        </section>

        <ArchMap />

        <ProjectWorkspace
          chaosMode={chaosMode}
          setChaosMode={setChaosMode}
          nodes={nodes}
          history={history}
        />

        <DeepDives />
        <Forum />
        <About />
        <Footer />
      </ErrorBoundary>
    </div>
  );
}
