"use client";

import { useEffect, useRef } from "react";

const NODES = [
  { x: 400, y: 60,  label: "Control Center", sub: "Next.js · React 19 · Tailwind v4", color: "#58a6ff" },
  { x: 60,  y: 200, label: "Container Engine", sub: "Rust · Linux · Namespaces · cgroups v2", color: "#3fb950" },
  { x: 400, y: 200, label: "Platform Nodes", sub: "Rust · LSM Storage · SWIM Gossip", color: "#8b5cf6" },
  { x: 700, y: 200, label: "Log Broker", sub: "Rust · mio · Lock-free SPSC", color: "#d2991d" },
  { x: 60,  y: 340, label: "Core Systems", sub: "Rust · SPSC Queue · Zero-alloc Logger", color: "#58a6ff" },
  { x: 400, y: 340, label: "Admin Tools", sub: "Rust · TUI · TCP Client · JSON Parser", color: "#8b5cf6" },
  { x: 700, y: 340, label: "Tauri Desktop", sub: "Rust · Tauri 1.5 · IPC Bridge", color: "#d2991d" },
];

const EDGES = [
  [0, 1], [0, 2], [0, 3], [1, 4], [2, 4],
  [2, 5], [0, 6],
];

export default function ArchMap() {
  const ref = useRef<HTMLCanvasElement>(null);

  useEffect(() => {
    const canvas = ref.current;
    if (!canvas) return;

    const ctx = canvas.getContext("2d");
    if (!ctx) return;

    // High-DPI Scaling setup
    const dpr = window.devicePixelRatio || 1;
    canvas.width = 800 * dpr;
    canvas.height = 420 * dpr;
    ctx.scale(dpr, dpr);

    let frame = 0;

    const draw = () => {
      ctx.clearRect(0, 0, 800, 420);

      const t = frame * 0.02;

      for (const [a, b] of EDGES) {
        const ax = NODES[a].x;
        const ay = NODES[a].y;
        const bx = NODES[b].x;
        const by = NODES[b].y;

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

      for (const node of NODES) {
        const pulse = 1 + Math.sin(t * 2 + node.x) * 0.08;

        ctx.beginPath();
        ctx.arc(node.x, node.y, 28 * pulse, 0, Math.PI * 2);
        ctx.fillStyle = "rgba(19, 22, 29, 0.95)";
        ctx.fill();
        ctx.strokeStyle = node.color;
        ctx.lineWidth = 1.5;
        ctx.stroke();

        ctx.beginPath();
        ctx.arc(node.x, node.y, 3, 0, Math.PI * 2);
        ctx.fillStyle = node.color;
        ctx.fill();

        ctx.font = "bold 11px 'JetBrains Mono', monospace";
        ctx.fillStyle = "#e1e4ea";
        ctx.textAlign = "center";
        ctx.fillText(node.label, node.x, node.y - 36);

        ctx.font = "9px 'JetBrains Mono', monospace";
        ctx.fillStyle = "#5c6270";
        ctx.fillText(node.sub, node.x, node.y - 20);
      }

      frame++;
      requestAnimationFrame(draw);
    };

    draw();
  }, []);

  return (
    <section id="arch" className="section">
      <div className="section-heading">System Architecture</div>
      <h2 className="section-title">How Everything Connects</h2>
      <p className="text-text-soft text-base max-w-2xl mb-8">
        Seven crates working together — from the kernel-level container runtime to
        the browser-based control center. Animated particles show data flow between
        components.
      </p>

      <div className="cyber-panel overflow-hidden">
        <canvas
          ref={ref}
          width={800}
          height={420}
          className="w-full h-auto"
        />
      </div>

      <div className="flex flex-wrap gap-3 mt-6 text-[11px] font-mono text-text-muted">
        {NODES.map((n) => (
          <span key={n.label} className="flex items-center gap-1.5">
            <span className="w-2 h-2 rounded-full" style={{ background: n.color }} />
            {n.label}
          </span>
        ))}
      </div>
    </section>
  );
}
