"use client";

import { ReactNode, cloneElement, isValidElement } from "react";
import { motion } from "framer-motion";
import { useMagneticCursor } from "../hooks/useMagneticCursor";

interface MagneticProps {
  children: ReactNode;
  strength?: number;
  radius?: number;
  className?: string;
}

export default function Magnetic({ children, strength = 0.25, radius = 80, className }: MagneticProps) {
  const { ref, style, onMouseLeave } = useMagneticCursor({ strength, radius });

  if (!isValidElement(children)) {
    return <span ref={ref as any} className={className}>{children}</span>;
  }

  return (
    <motion.div
      ref={ref as any}
      style={style}
      className={className}
      onMouseLeave={onMouseLeave}
    >
      {children}
    </motion.div>
  );
}
