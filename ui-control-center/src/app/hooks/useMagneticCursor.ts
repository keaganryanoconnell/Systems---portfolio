"use client";

import { useRef, useEffect, useCallback } from "react";
import { useMotionValue, useSpring, type SpringOptions } from "framer-motion";

interface MagneticOptions {
  strength?: number;
  radius?: number;
}

export function useMagneticCursor(options: MagneticOptions = {}) {
  const { strength = 0.3, radius = 80 } = options;
  const ref = useRef<HTMLElement>(null);
  const x = useMotionValue(0);
  const y = useMotionValue(0);

  const springConfig: SpringOptions = { stiffness: 150, damping: 15, mass: 0.1 };
  const springX = useSpring(x, springConfig);
  const springY = useSpring(y, springConfig);

  const handleMouseMove = useCallback(
    (e: MouseEvent) => {
      const el = ref.current;
      if (!el) return;
      const rect = el.getBoundingClientRect();
      const centerX = rect.left + rect.width / 2;
      const centerY = rect.top + rect.height / 2;
      const distX = e.clientX - centerX;
      const distY = e.clientY - centerY;
      const distance = Math.sqrt(distX * distX + distY * distY);

      if (distance < radius) {
        const pull = (1 - distance / radius) * strength;
        x.set(distX * pull);
        y.set(distY * pull);
      } else {
        x.set(0);
        y.set(0);
      }
    },
    [radius, strength, x, y]
  );

  const handleMouseLeave = useCallback(() => {
    x.set(0);
    y.set(0);
  }, [x, y]);

  useEffect(() => {
    window.addEventListener("mousemove", handleMouseMove);
    return () => window.removeEventListener("mousemove", handleMouseMove);
  }, [handleMouseMove]);

  return {
    ref,
    style: { x: springX, y: springY } as const,
    onMouseLeave: handleMouseLeave,
  };
}
