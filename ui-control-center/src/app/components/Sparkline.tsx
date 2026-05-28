"use client";

import { useEffect, useRef } from "react";

interface SparklineProps {
  data: number[];
  width?: number;
  height?: number;
  color?: 'red' | 'blue' | 'amber' | 'green';
}

export default function Sparkline({ data, width = 120, height = 30, color = 'green' }: SparklineProps) {
  const canvasRef = useRef<HTMLCanvasElement>(null);

  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;

    const ctx = canvas.getContext("2d");
    if (!ctx) return;

    ctx.clearRect(0, 0, width, height);

    if (data.length < 2) return;

    const strokeColors = {
      green: "#3fb950",
      amber: "#d2991d",
      red: "#f85149",
      blue: "#58a6ff"
    };

    const fillColors = {
      green: "rgba(63, 185, 80, 0.08)",
      amber: "rgba(210, 153, 29, 0.08)",
      red: "rgba(248, 81, 73, 0.08)",
      blue: "rgba(88, 166, 255, 0.08)"
    };

    const colorHex = strokeColors[color];
    const fillHex = fillColors[color];

    const max = Math.max(...data);
    const min = Math.min(...data);
    const range = max - min === 0 ? 1 : max - min;

    ctx.beginPath();
    ctx.strokeStyle = colorHex;
    ctx.lineWidth = 1.5;
    ctx.lineCap = "round";
    ctx.lineJoin = "round";

    const step = width / (data.length - 1);

    data.forEach((val, i) => {
      const x = i * step;
      const y = height - 2 - ((val - min) / range) * (height - 4);
      if (i === 0) {
        ctx.moveTo(x, y);
      } else {
        ctx.lineTo(x, y);
      }
    });
    ctx.stroke();

    ctx.lineTo((data.length - 1) * step, height);
    ctx.lineTo(0, height);
    ctx.closePath();
    ctx.fillStyle = fillHex;
    ctx.fill();

  }, [data, width, height, color]);

  return (
    <canvas 
      ref={canvasRef} 
      width={width} 
      height={height} 
      className="block"
      style={{ width, height }}
    />
  );
}
