"use client";

import { useEffect, useRef, useState } from "react";
import { NodeTelemetry } from "../utils/tauri";
import Sparkline from "./Sparkline";

interface ViewClusterNodesProps {
  nodes: NodeTelemetry[];
  history: {
    cpu: Record<number, number[]>;
    memory: Record<number, number[]>;
    fd: Record<number, number[]>;
  };
  chaosMode: {
    partitionSplit: boolean;
    malformedFrames: boolean;
    crashNode2: boolean;
    fuzzerRunning: boolean;
  };
}

export default function ViewClusterNodes({ nodes, history, chaosMode }: ViewClusterNodesProps) {
  const mapCanvasRef = useRef<HTMLCanvasElement>(null);
  const containerRef = useRef<HTMLDivElement>(null);
  const [dimensions, setDimensions] = useState({ width: 800, height: 320 });

  // Handle resizing of the connector map canvas
  useEffect(() => {
    if (!containerRef.current) return;
    const observer = new ResizeObserver((entries) => {
      for (let entry of entries) {
        setDimensions({
          width: Math.floor(entry.contentRect.width),
          height: Math.floor(entry.contentRect.height || 320),
        });
      }
    });
    observer.observe(containerRef.current);
    return () => observer.disconnect();
  }, []);

  // Consensus Replication Routing Map Canvas Animator
  useEffect(() => {
    const canvas = mapCanvasRef.current;
    if (!canvas) return;

    const ctx = canvas.getContext("2d");
    if (!ctx) return;

    let animationId: number;
    let pulseProgress = 0;

    const renderMap = () => {
      const { width, height } = dimensions;
      ctx.clearRect(0, 0, width, height);

      // Draw background grid lines inside map
      ctx.strokeStyle = "rgba(180, 76, 255, 0.03)";
      ctx.lineWidth = 1;
      for (let x = 0; x < width; x += 40) {
        ctx.beginPath();
        ctx.moveTo(x, 0);
        ctx.lineTo(x, height);
        ctx.stroke();
      }
      for (let y = 0; y < height; y += 40) {
        ctx.beginPath();
        ctx.moveTo(0, y);
        ctx.lineTo(width, y);
        ctx.stroke();
      }

      // Gateway coordinates (Left side)
      const gwX = 100;
      const gwY = height / 2;

      // Define Node layout coordinates (distributed on the right side)
      const nodeX = width - 120;
      const nodeYSpacing = height / (nodes.length + 1);

      // Draw Client Gateway (Source node)
      ctx.beginPath();
      ctx.arc(gwX, gwY, 12, 0, Math.PI * 2);
      ctx.fillStyle = "#020204";
      ctx.fill();
      ctx.strokeStyle = "#00e5ff"; // Cyber Cyan
      ctx.lineWidth = 2.5;
      ctx.stroke();
      
      // Gateway pulse ring
      ctx.beginPath();
      ctx.arc(gwX, gwY, 12 + (pulseProgress * 15) % 25, 0, Math.PI * 2);
      ctx.strokeStyle = `rgba(0, 229, 255, ${Math.max(0, 1 - pulseProgress * 0.5)})`;
      ctx.lineWidth = 1;
      ctx.stroke();

      ctx.fillStyle = "#ffffff";
      ctx.font = "bold 10px var(--font-mono)";
      ctx.textAlign = "center";
      ctx.fillText("GATEWAY", gwX, gwY - 20);
      ctx.fillStyle = "#00e5ff";
      ctx.font = "8px var(--font-mono)";
      ctx.fillText("CLIENT_API", gwX, gwY + 24);

      // Loop over nodes and draw connectors + node visual representations
      nodes.forEach((node, index) => {
        const nX = nodeX;
        const nY = nodeYSpacing * (index + 1);

        const isOffline = node.status === 'Offline';
        const isDegraded = node.status === 'Degraded';

        // Set colors based on status
        let strokeColor = "#39ff14"; // Healthy: Emerald Green
        let pulseColor = "rgba(57, 255, 20, 0.95)";
        let lineStyle: "solid" | "dashed" = "solid";

        if (isOffline) {
          strokeColor = "#ff2d95"; // Offline: Crimson Red
          pulseColor = "rgba(0, 0, 0, 0)";
          lineStyle = "dashed";
        } else if (isDegraded) {
          strokeColor = "#ffe600"; // Degraded/Lagging: Amber Yellow
          pulseColor = "rgba(255, 230, 0, 0.7)";
          lineStyle = "dashed";
        }

        // Draw Connector Path (Symmetric Bezier Curve)
        ctx.beginPath();
        ctx.moveTo(gwX + 12, gwY);
        // Control points for a smooth curves
        ctx.bezierCurveTo(
          (gwX + nX) / 2, gwY,
          (gwX + nX) / 2, nY,
          nX - 15, nY
        );

        ctx.strokeStyle = isOffline 
          ? "rgba(255, 45, 149, 0.15)" 
          : isDegraded 
            ? "rgba(255, 230, 0, 0.25)" 
            : "rgba(57, 255, 20, 0.35)";
            
        ctx.lineWidth = isOffline ? 1 : 1.5;
        if (lineStyle === "dashed") {
          ctx.setLineDash([4, 4]);
        } else {
          ctx.setLineDash([]);
        }
        ctx.stroke();
        ctx.setLineDash([]); // Reset line dash

        // Draw pulse traveling along the Bezier curve
        if (!isOffline && (!chaosMode.partitionSplit || index < 2)) {
          // Calculate point on Bezier Curve
          const t = (pulseProgress + index * 0.2) % 1.0;
          const cp1x = (gwX + nX) / 2;
          const cp1y = gwY;
          const cp2x = (gwX + nX) / 2;
          const cp2y = nY;

          // Cubic Bezier interpolation formula
          const mt = 1 - t;
          const x = mt * mt * mt * (gwX + 12) + 
                    3 * mt * mt * t * cp1x + 
                    3 * mt * t * t * cp2x + 
                    t * t * t * (nX - 15);
                    
          const y = mt * mt * mt * gwY + 
                    3 * mt * mt * t * cp1y + 
                    3 * mt * t * t * cp2y + 
                    t * t * t * nY;

          ctx.beginPath();
          ctx.arc(x, y, 4, 0, Math.PI * 2);
          ctx.fillStyle = pulseColor;
          ctx.shadowBlur = 8;
          ctx.shadowColor = strokeColor;
          ctx.fill();
          ctx.shadowBlur = 0; // Reset
        }

        // Draw cluster node node visualizer
        ctx.beginPath();
        ctx.arc(nX, nY, 15, 0, Math.PI * 2);
        ctx.fillStyle = "#0d0221";
        ctx.fill();
        ctx.strokeStyle = strokeColor;
        ctx.lineWidth = 2.5;
        ctx.stroke();

        // Node ID tag inside node circle
        ctx.fillStyle = isOffline ? "#ff2d95" : "#ffffff";
        ctx.font = "bold 10px var(--font-mono)";
        ctx.textAlign = "center";
        ctx.fillText(`N${node.nodeId}`, nX, nY + 3.5);

        // Node Meta descriptors
        ctx.font = "bold 9px var(--font-mono)";
        ctx.textAlign = "left";
        ctx.fillStyle = isOffline ? "#ff2d95" : isDegraded ? "#ffe600" : "#ffffff";
        ctx.fillText(node.role, nX + 24, nY - 4);
        
        ctx.font = "8px var(--font-mono)";
        ctx.fillStyle = "rgba(255, 255, 255, 0.4)";
        const statusText = isOffline ? "SIGKILL_HALT" : isDegraded ? `REPL_LAG (${node.replicationLag}ms)` : `RTT (${(0.5 + index * 1.5).toFixed(1)}ms)`;
        ctx.fillText(statusText, nX + 24, nY + 7);
      });

      // Consensus quorum badge (bottom middle)
      ctx.fillStyle = "#0d0221";
      ctx.strokeStyle = "#b44cff";
      ctx.lineWidth = 1;
      const badgeW = 220;
      const badgeH = 24;
      const badgeX = (width - badgeW) / 2;
      const badgeY = height - 35;
      
      ctx.beginPath();
      ctx.roundRect?.(badgeX, badgeY, badgeW, badgeH, 4);
      ctx.fill();
      ctx.stroke();

      ctx.fillStyle = chaosMode.partitionSplit ? "#ffe600" : "#39ff14";
      ctx.font = "bold 8px var(--font-mono)";
      ctx.textAlign = "center";
      const quorumText = chaosMode.partitionSplit 
        ? "QUORUM STATE: PARTITION_SPLIT_BRAIN (3/2+1 DEAD)"
        : `QUORUM STATUS: OPTIMAL (3/2 + 1 NODES SYNCED)`;
      ctx.fillText(quorumText, width / 2, badgeY + 15);

      pulseProgress += 0.005;
      if (pulseProgress > 1.0) pulseProgress = 0;
      animationId = requestAnimationFrame(renderMap);
    };

    renderMap();
    return () => cancelAnimationFrame(animationId);
  }, [dimensions, nodes, chaosMode]);

  return (
    <div className="flex flex-col gap-6">
      {/* Component B: Replication Routing Map */}
      <div className="cyber-panel p-4 rounded" ref={containerRef}>
        <div className="flex items-center justify-between border-b border-border pb-2 mb-2">
          <span className="text-xs font-mono font-bold text-text-soft tracking-wider">CONSENSUS_REPLICATION_MAP // SWIM_RAFT_HEURISTICS</span>
          <div className="flex gap-4">
            <span className="text-[10px] font-mono text-green font-bold">● ACTIVE REPLICATION ROUTE</span>
            <span className="text-[10px] font-mono text-gold font-bold">▲ DEGRADED PATHWAY</span>
          </div>
        </div>
        <div className="w-full min-h-[180px] h-[220px] relative bg-bg rounded overflow-hidden">
          <canvas 
            ref={mapCanvasRef} 
            width={dimensions.width} 
            height={dimensions.height} 
            className="absolute inset-0 w-full h-full block" 
          />
        </div>
      </div>

      {/* Components A & C: Node Topology Cards & Metrics */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-5 gap-4">
        {nodes.map((node) => {
          const isOffline = node.status === 'Offline';
          const isLeader = node.role === 'Leader';
          const isCandidate = node.role === 'Candidate';

          // Node history references
          const cpuHistory = history.cpu[node.nodeId] || [0];
          const memoryHistory = history.memory[node.nodeId] || [0];
          const fdHistory = history.fd[node.nodeId] || [0];

          return (
            <div 
              key={node.nodeId} 
              className={`cyber-panel p-4 rounded transition-all duration-300 ${
                isOffline 
                  ? "border-red-border opacity-60" 
                  : isLeader 
                    ? "border-blue-border hover:border-blue/60" 
                    : "hover:border-green/40"
              }`}
            >
              {/* Card Header */}
              <div className="flex items-center justify-between border-b border-border pb-2.5 mb-3">
                <div className="flex items-center gap-2">
                  <div className={`text-xs font-mono font-bold px-2 py-0.5 rounded ${
                    isLeader 
                      ? "bg-blue-bg text-blue border border-blue-border" 
                      : isCandidate
                        ? "bg-gold-bg text-gold border border-gold-border"
                        : "bg-surface text-text-soft"
                  }`}>
                    NODE_0{node.nodeId}
                  </div>
                </div>
                {/* Status indicator */}
                <div className="flex items-center gap-1.5">
                  <span className={`h-1.5 w-1.5 rounded-full inline-block ${
                    node.status === 'Healthy' 
                      ? "bg-green animate-pulse" 
                      : node.status === 'Degraded' 
                        ? "bg-gold animate-pulse" 
                        : "bg-red"
                  }`}></span>
                  <span className={`text-[8px] font-mono font-bold ${
                    node.status === 'Healthy' 
                      ? "text-green" 
                      : node.status === 'Degraded' 
                        ? "text-gold" 
                        : "text-red"
                  }`}>
                    {node.status}
                  </span>
                </div>
              </div>

              {/* Node Role/Metadata Mini Badge list */}
              <div className="flex flex-wrap gap-1.5 mb-4">
                <span className="text-[8px] font-mono font-bold bg-[#14161f] px-1.5 py-0.5 rounded text-text-soft">
                  {node.role}
                </span>
                <span className="text-[8px] font-mono font-bold bg-[#14161f] px-1.5 py-0.5 rounded text-text-soft">
                  PORT: {4000 + node.nodeId}
                </span>
              </div>

              {/* Component C: Sparklines & Core Metrics */}
              <div className="space-y-4">
                {/* CPU load */}
                <div className="flex flex-col gap-1">
                  <div className="flex items-center justify-between text-[9px] font-mono">
                    <span className="text-text-soft font-bold">CPU LOAD:</span>
                    <span className="text-text font-bold">{node.cpu}%</span>
                  </div>
                  <div className="flex items-center justify-between gap-2">
                    <div className="flex-1 bg-bg h-1.5 rounded overflow-hidden relative">
                      <div 
                        className="bg-green h-full rounded transition-all duration-300"
                        style={{ width: `${node.cpu}%` }}
                      />
                    </div>
                    <div className="opacity-85">
                      <Sparkline data={cpuHistory} color="green" width={60} height={16} />
                    </div>
                  </div>
                </div>

                {/* Memory Allocation */}
                <div className="flex flex-col gap-1">
                  <div className="flex items-center justify-between text-[9px] font-mono">
                    <span className="text-text-soft font-bold">ARENA MEMORY:</span>
                    <span className="text-text font-bold">{node.arenaMemoryAllocated}MB / {node.arenaMemoryTotal}MB</span>
                  </div>
                  <div className="flex items-center justify-between gap-2">
                    <div className="flex-1 bg-bg h-1.5 rounded overflow-hidden relative">
                      <div 
                        className="bg-gold h-full rounded transition-all duration-300"
                        style={{ width: `${(node.arenaMemoryAllocated / (node.arenaMemoryTotal || 1)) * 100}%` }}
                      />
                    </div>
                    <div className="opacity-85">
                      <Sparkline data={memoryHistory} color="amber" width={60} height={16} />
                    </div>
                  </div>
                </div>

                {/* File Descriptor Pool */}
                <div className="flex flex-col gap-1">
                  <div className="flex items-center justify-between text-[9px] font-mono">
                    <span className="text-text-soft font-bold">FD_POOL LOCKS:</span>
                    <span className="text-text font-bold">{node.activeFdPool} active</span>
                  </div>
                  <div className="flex items-center justify-between gap-2">
                    <div className="flex-1 bg-bg h-1.5 rounded overflow-hidden relative">
                      <div 
                        className="bg-blue h-full rounded transition-all duration-300"
                        style={{ width: `${(node.activeFdPool / 200) * 100}%` }}
                      />
                    </div>
                    <div className="opacity-85">
                      <Sparkline data={fdHistory} color="blue" width={60} height={16} />
                    </div>
                  </div>
                </div>

                {/* Bottom detail specs */}
                <div className="grid grid-cols-2 gap-2 border-t border-border/60 pt-2 text-[9px] font-mono">
                  <div>
                    <span className="text-text-soft block">IOPS THROUGH:</span>
                    <span className="text-text font-bold block mt-0.5">
                      {node.iops.toLocaleString()}
                    </span>
                  </div>
                  <div>
                    <span className="text-text-soft block">STORAGE LSM:</span>
                    <span className="text-text font-bold block mt-0.5 text-ellipsis overflow-hidden">
                      {(Number(node.lsmStorageBytes) / 1e9).toFixed(2)} GB
                    </span>
                  </div>
                </div>
              </div>
            </div>
          );
        })}
      </div>
    </div>
  );
}
