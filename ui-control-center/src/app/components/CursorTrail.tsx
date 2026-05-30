"use client";

import { useEffect, useRef } from "react";
import { useMotionValue } from "framer-motion";

const TRAIL_LENGTH = 15;

export default function CursorTrail() {
  const positions = useRef<{ x: number; y: number; age: number }[]>(
    Array.from({ length: TRAIL_LENGTH }, () => ({ x: 0, y: 0, age: 0 }))
  );
  const mouseX = useMotionValue(-100);
  const mouseY = useMotionValue(-100);
  const dotsRef = useRef<(HTMLDivElement | null)[]>([]);

  useEffect(() => {
    const isTouch = matchMedia("(hover: none)").matches;
    if (isTouch) return;

    let frame: number;

    const move = (e: MouseEvent) => {
      mouseX.set(e.clientX);
      mouseY.set(e.clientY);
    };

    const update = () => {
      const pos = positions.current;
      const mx = mouseX.get();
      const my = mouseY.get();

      // Shift array
      for (let i = TRAIL_LENGTH - 1; i > 0; i--) {
        pos[i].x = pos[i - 1].x;
        pos[i].y = pos[i - 1].y;
        pos[i].age = pos[i - 1].age + 1;
      }
      pos[0].x = mx;
      pos[0].y = my;
      pos[0].age = 0;

      for (let i = 0; i < TRAIL_LENGTH; i++) {
        const dot = dotsRef.current[i];
        if (!dot) continue;
        const alpha = Math.max(0, 1 - pos[i].age / TRAIL_LENGTH);
        const scale = Math.max(0.1, 1 - pos[i].age / TRAIL_LENGTH);
        dot.style.transform = `translate(${pos[i].x}px, ${pos[i].y}px) scale(${scale})`;
        dot.style.opacity = String(alpha * 0.6);
      }

      frame = requestAnimationFrame(update);
    };

    window.addEventListener("mousemove", move, { passive: true });
    frame = requestAnimationFrame(update);

    return () => {
      window.removeEventListener("mousemove", move);
      cancelAnimationFrame(frame);
    };
  }, [mouseX, mouseY]);

  return (
    <div className="fixed inset-0 pointer-events-none z-[199] overflow-hidden">
      {Array.from({ length: TRAIL_LENGTH }).map((_, i) => (
        <div
          key={i}
          ref={(el) => { dotsRef.current[i] = el; }}
          className="absolute rounded-full bg-[#d2991d]"
          style={{
            width: i === 0 ? 4 : 3,
            height: i === 0 ? 4 : 3,
            translate: "-50% -50%",
            boxShadow: "0 0 4px rgba(210,153,29,0.3)",
          }}
        />
      ))}
    </div>
  );
}
