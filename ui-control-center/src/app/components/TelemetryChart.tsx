"use client";

import { useEffect, useRef, useState } from "react";

interface TelemetryChartProps {
  cpuHistory: number[];
  iopsHistory: number[];
  memoryHistory: number[];
}

export default function TelemetryChart({ cpuHistory, iopsHistory, memoryHistory }: TelemetryChartProps) {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const containerRef = useRef<HTMLDivElement>(null);
  const [dimensions, setDimensions] = useState({ width: 600, height: 300 });

  useEffect(() => {
    if (!containerRef.current) return;
    
    const resizeObserver = new ResizeObserver((entries) => {
      for (let entry of entries) {
        setDimensions({
          width: Math.floor(entry.contentRect.width),
          height: Math.floor(entry.contentRect.height),
        });
      }
    });

    resizeObserver.observe(containerRef.current);
    return () => resizeObserver.disconnect();
  }, []);

  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;

    const ctx = canvas.getContext("2d");
    if (!ctx) return;

    const { width, height } = dimensions;
    ctx.clearRect(0, 0, width, height);

    // Draw soft grid lines
    ctx.strokeStyle = "rgba(180, 76, 255, 0.05)";
    ctx.lineWidth = 1;
    
    const gridSpacingX = 40;
    for (let x = 0; x < width; x += gridSpacingX) {
      ctx.beginPath();
      ctx.moveTo(x, 0);
      ctx.lineTo(x, height);
      ctx.stroke();
    }

    const gridSpacingY = 30;
    for (let y = 0; y < height; y += gridSpacingY) {
      ctx.beginPath();
      ctx.moveTo(0, y);
      ctx.lineTo(width, y);
      ctx.stroke();
    }

    const drawLine = (data: number[], color: string, shadowColor: string, maxVal: number) => {
      if (data.length < 2) return;
      
      ctx.beginPath();
      ctx.strokeStyle = color;
      ctx.lineWidth = 2.5;
      ctx.lineCap = "round";
      ctx.lineJoin = "round";

      ctx.shadowBlur = 6;
      ctx.shadowColor = shadowColor;

      const stepX = width / (data.length - 1);

      data.forEach((val, index) => {
        const x = index * stepX;
        const ratio = maxVal > 0 ? val / maxVal : 0;
        const y = height - 10 - ratio * (height - 30);
        if (index === 0) {
          ctx.moveTo(x, y);
        } else {
          ctx.lineTo(x, y);
        }
      });
      ctx.stroke();
      ctx.shadowBlur = 0;

      ctx.beginPath();
      data.forEach((val, index) => {
        const x = index * stepX;
        const ratio = maxVal > 0 ? val / maxVal : 0;
        const y = height - 10 - ratio * (height - 30);
        if (index === 0) {
          ctx.moveTo(x, y);
        } else {
          ctx.lineTo(x, y);
        }
      });
      ctx.lineTo((data.length - 1) * stepX, height);
      ctx.lineTo(0, height);
      ctx.closePath();
      
      const gradient = ctx.createLinearGradient(0, 0, 0, height);
      gradient.addColorStop(0, shadowColor.replace("0.3", "0.06"));
      gradient.addColorStop(1, "rgba(180, 76, 255, 0)");
      ctx.fillStyle = gradient;
      ctx.fill();
    };

    drawLine(cpuHistory, "#39ff14", "rgba(57, 255, 20, 0.3)", 100);

    const maxIops = Math.max(...iopsHistory, 20000);
    drawLine(iopsHistory, "#00e5ff", "rgba(0, 229, 255, 0.3)", maxIops);

    drawLine(memoryHistory, "#ffe600", "rgba(255, 230, 0, 0.3)", 1024);

  }, [dimensions, cpuHistory, iopsHistory, memoryHistory]);

  return (
    <div className="w-full h-full flex flex-col gap-2">
      <div className="flex flex-wrap items-center justify-between gap-4 border-b border-border pb-2 px-1">
        <span className="text-xs font-mono font-bold text-text-soft tracking-wider">CLUSTER_METRICS // REAL_TIME_STREAM</span>
        <div className="flex gap-4">
          <div className="flex items-center gap-1.5">
            <span className="w-2 h-2 rounded-full bg-green inline-block"></span>
            <span className="text-[10px] font-mono font-bold text-green">CPU LOAD (AVG %)</span>
          </div>
          <div className="flex items-center gap-1.5">
            <span className="w-2 h-2 rounded-full bg-blue inline-block"></span>
            <span className="text-[10px] font-mono font-bold text-blue">IOPS LOAD</span>
          </div>
          <div className="flex items-center gap-1.5">
            <span className="w-2 h-2 rounded-full bg-gold inline-block"></span>
            <span className="text-[10px] font-mono font-bold text-gold">ARENA MEMORY (MB)</span>
          </div>
        </div>
      </div>
      <div ref={containerRef} className="flex-1 bg-surface rounded border border-border overflow-hidden relative">
        <canvas 
          ref={canvasRef} 
          width={dimensions.width} 
          height={dimensions.height} 
          className="absolute inset-0 w-full h-full"
        />
      </div>
    </div>
  );
}
