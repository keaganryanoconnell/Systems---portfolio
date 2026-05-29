"use client";

import { useEffect, useRef, useState } from "react";

interface NodeSpec {
  label: string;
  sub: string;
  color: string;
}

const NODE_SPECS: NodeSpec[] = [
  { label: "Control Center", sub: "Next.js · React 19 · Tailwind v4", color: "#58a6ff" },
  { label: "Container Engine", sub: "Rust · Linux · Namespaces · cgroups v2", color: "#3fb950" },
  { label: "Platform Nodes", sub: "Rust · LSM Storage · SWIM Gossip", color: "#8b5cf6" },
  { label: "Log Broker", sub: "Rust · mio · Lock-free SPSC", color: "#d2991d" },
  { label: "Core Systems", sub: "Rust · SPSC Queue · Zero-alloc Logger", color: "#58a6ff" },
  { label: "Admin Tools", sub: "Rust · TUI · TCP Client · JSON Parser", color: "#8b5cf6" },
  { label: "Compute Orchestrator", sub: "Rust · Actor Model · OpenTelemetry", color: "#3fb950" },
  { label: "Tauri Desktop", sub: "Rust · Tauri 1.5 · IPC Bridge", color: "#d2991d" },
];

const EDGES: [number, number][] = [
  [0, 1], [0, 2], [0, 3], [1, 4], [2, 4],
  [2, 5], [0, 7], [0, 6], [6, 2],
];

const CANVAS_W = 800;
const CANVAS_H = 420;

function layoutNodes(count: number): { x: number; y: number }[] {
  const positions: { x: number; y: number }[] = [];
  positions.push({ x: CANVAS_W / 2, y: 50 });
  const cols = 3;
  const rowH = 120;
  const colW = (CANVAS_W - 100) / cols;
  const startX = colW / 2 + 30;
  for (let i = 2; i <= count + 1; i++) {
    const row = Math.floor((i - 2) / cols);
    const col = (i - 2) % cols;
    positions.push({
      x: startX + col * colW,
      y: 150 + row * rowH,
    });
  }
  return positions;
}

export default function ArchMap() {
  const ref = useRef<HTMLCanvasElement>(null);
  const containerRef = useRef<HTMLDivElement>(null);
  const [size, setSize] = useState({ w: 800, h: 420 });

  useEffect(() => {
    if (!containerRef.current) return;
    const ro = new ResizeObserver((entries) => {
      for (const entry of entries) {
        setSize({
          w: Math.floor(entry.contentRect.width),
          h: Math.floor(entry.contentRect.width * (CANVAS_H / CANVAS_W)),
        });
      }
    });
    ro.observe(containerRef.current);
    return () => ro.disconnect();
  }, []);

  useEffect(() => {
    const canvas = ref.current;
    if (!canvas) return;
    const ctx = canvas.getContext("2d");
    if (!ctx) return;

    const dpr = window.devicePixelRatio || 1;
    canvas.width = size.w * dpr;
    canvas.height = size.h * dpr;
    canvas.style.width = `${size.w}px`;
    canvas.style.height = `${size.h}px`;
    ctx.scale(dpr * (size.w / CANVAS_W), dpr * (size.h / CANVAS_H));

    let frame = 0;
    let animId: number;

    const positions = layoutNodes(NODE_SPECS.length);

    const draw = () => {
      ctx.clearRect(0, 0, CANVAS_W, CANVAS_H);
      const t = frame * 0.02;

      for (const [a, b] of EDGES) {
        const ax = positions[a].x;
        const ay = positions[a].y;
        const bx = positions[b].x;
        const by = positions[b].y;

        ctx.beginPath();
        ctx.moveTo(ax, ay);
        ctx.lineTo(bx, by);
        ctx.strokeStyle = "rgba(88, 166, 255, 0.1)";
        ctx.lineWidth = 1;
        ctx.stroke();

        const px = ax + (bx - ax) * ((Math.sin(t * 1.5 + a) + 1) / 2);
        const py = ay + (by - ay) * ((Math.cos(t * 1.7 + b) + 1) / 2);
        ctx.beginPath();
        ctx.arc(px, py, 2.5, 0, Math.PI * 2);
        ctx.fillStyle = `rgba(210, 153, 29, ${0.3 + Math.sin(t * 2 + a + b) * 0.2})`;
        ctx.fill();
      }

      for (let i = 0; i < NODE_SPECS.length; i++) {
        const node = NODE_SPECS[i];
        const pos = positions[i];
        const pulse = 1 + Math.sin(t * 2 + pos.x) * 0.08;

        ctx.beginPath();
        ctx.arc(pos.x, pos.y, 28 * pulse, 0, Math.PI * 2);
        ctx.fillStyle = "rgba(19, 22, 29, 0.95)";
        ctx.fill();
        ctx.strokeStyle = node.color;
        ctx.lineWidth = 1.5;
        ctx.stroke();

        ctx.beginPath();
        ctx.arc(pos.x, pos.y, 3, 0, Math.PI * 2);
        ctx.fillStyle = node.color;
        ctx.fill();

        ctx.font = "bold 11px 'JetBrains Mono', monospace";
        ctx.fillStyle = "#e1e4ea";
        ctx.textAlign = "center";
        ctx.fillText(node.label, pos.x, pos.y - 36);

        ctx.font = "9px 'JetBrains Mono', monospace";
        ctx.fillStyle = "#5c6270";
        ctx.fillText(node.sub, pos.x, pos.y - 20);
      }

      frame++;
      animId = requestAnimationFrame(draw);
    };

    draw();
    return () => cancelAnimationFrame(animId);
  }, [size]);

  return (
    <section id="arch" className="section">
      <div className="section-heading">System Architecture</div>
      <h2 className="section-title">How Everything Connects</h2>
      <p className="text-text-soft text-base max-w-2xl mb-8">
        Eight crates working together — from the kernel-level container runtime to
        the browser-based control plane. Animated particles show data flow between
        components.
      </p>

      <div ref={containerRef} className="cyber-panel overflow-hidden">
        <canvas ref={ref} className="w-full block" />
      </div>

      <div className="flex flex-wrap gap-3 mt-6 text-[11px] font-mono text-text-muted">
        {NODE_SPECS.map((n) => (
          <span key={n.label} className="flex items-center gap-1.5">
            <span className="w-2 h-2 rounded-full" style={{ background: n.color }} />
            {n.label}
          </span>
        ))}
      </div>
    </section>
  );
}
