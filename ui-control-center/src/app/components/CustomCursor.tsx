"use client";

import { useEffect, useRef } from "react";
import { motion, useMotionValue, useSpring } from "framer-motion";

export default function CustomCursor() {
  const mouseX = useMotionValue(-100);
  const mouseY = useMotionValue(-100);
  const springX = useSpring(mouseX, { stiffness: 500, damping: 28, mass: 0.5 });
  const springY = useSpring(mouseY, { stiffness: 500, damping: 28, mass: 0.5 });
  const ringX = useSpring(mouseX, { stiffness: 150, damping: 20, mass: 1.2 });
  const ringY = useSpring(mouseY, { stiffness: 150, damping: 20, mass: 1.2 });
  const isHovering = useRef(false);
  const [hovering, setHovering] = useState(false);

  useEffect(() => {
    const isTouch = matchMedia("(hover: none)").matches;
    if (isTouch) return;

    const move = (e: MouseEvent) => {
      mouseX.set(e.clientX);
      mouseY.set(e.clientY);
    };

    const over = (e: MouseEvent) => {
      const target = e.target as HTMLElement;
      if (
        target.tagName === "A" ||
        target.tagName === "BUTTON" ||
        target.closest("a") ||
        target.closest("button") ||
        target.closest(".cyber-panel") ||
        target.closest("[data-magnetic]")
      ) {
        isHovering.current = true;
        setHovering(true);
      }
    };

    const out = () => {
      isHovering.current = false;
      setHovering(false);
    };

    window.addEventListener("mousemove", move, { passive: true });
    document.addEventListener("mouseover", over, { passive: true });
    document.addEventListener("mouseout", out, { passive: true });

    return () => {
      window.removeEventListener("mousemove", move);
      document.removeEventListener("mouseover", over);
      document.removeEventListener("mouseout", out);
    };
  }, [mouseX, mouseY]);

  const ringSize = hovering ? 48 : 24;
  const dotSize = hovering ? 6 : 8;
  const ringColor = hovering ? "#58a6ff" : "#d2991d";

  return (
    <>
      <motion.div
        className="fixed pointer-events-none z-[200] rounded-full"
        style={{
          x: ringX,
          y: ringY,
          width: ringSize,
          height: ringSize,
          border: `1.5px solid ${ringColor}`,
          translateX: "-50%",
          translateY: "-50%",
          opacity: 0.7,
        }}
        animate={{ rotate: 360 }}
        transition={{ duration: 8, repeat: Infinity, ease: "linear" }}
      />
      <motion.div
        className="fixed pointer-events-none z-[201] rounded-full bg-[#ffd700]"
        style={{
          x: springX,
          y: springY,
          width: dotSize,
          height: dotSize,
          translateX: "-50%",
          translateY: "-50%",
          boxShadow: hovering
            ? "0 0 12px rgba(88,166,255,0.6)"
            : "0 0 8px rgba(210,153,29,0.5)",
        }}
      />
    </>
  );
}

import { useState } from "react";
