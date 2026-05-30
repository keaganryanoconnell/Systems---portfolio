"use client";

import { ReactNode, useEffect } from "react";
import WebGLProvider from "./WebGLProvider";
import CustomCursor from "./CustomCursor";
import CursorTrail from "./CursorTrail";
import CommandPalette from "./CommandPalette";

export default function ClientLayout({ children }: { children: ReactNode }) {
  useEffect(() => {
    const isTouch = matchMedia("(hover: none)").matches;
    if (!isTouch) {
      document.documentElement.classList.add("custom-cursor");
    }
  }, []);

  return (
    <WebGLProvider>
      <CommandPalette />
      <CustomCursor />
      <CursorTrail />
      {children}
    </WebGLProvider>
  );
}
