"use client";

import { createContext, useContext, useRef, useCallback, useEffect, useState, type ReactNode } from "react";
import * as THREE from "three";

interface WebGLContextValue {
  renderer: THREE.WebGLRenderer | null;
  claim: (owner: string, animate: (dt: number) => void) => void;
  release: (owner: string) => void;
  addScene: (owner: string, scene: THREE.Scene, camera: THREE.Camera) => void;
  removeScene: (owner: string) => void;
}

const WebGLContext = createContext<WebGLContextValue>({
  renderer: null,
  claim: () => {},
  release: () => {},
  addScene: () => {},
  removeScene: () => {},
});

export function useWebGL() {
  return useContext(WebGLContext);
}

export default function WebGLProvider({ children }: { children: ReactNode }) {
  const rendererRef = useRef<THREE.WebGLRenderer | null>(null);
  const canvasRef = useRef<HTMLCanvasElement | null>(null);
  const ownerRef = useRef<string | null>(null);
  const animateRef = useRef<((dt: number) => void) | null>(null);
  const scenesRef = useRef<Map<string, { scene: THREE.Scene; camera: THREE.Camera }>>(new Map());
  const rafRef = useRef<number>(0);
  const clockRef = useRef(new THREE.Clock());
  const [mounted, setMounted] = useState(false);

  useEffect(() => {
    if (!canvasRef.current) return;

    const renderer = new THREE.WebGLRenderer({
      canvas: canvasRef.current,
      alpha: true,
      antialias: true,
      powerPreference: "high-performance",
    });
    renderer.setPixelRatio(Math.min(window.devicePixelRatio, 2));
    renderer.setSize(window.innerWidth, window.innerHeight);
    renderer.autoClear = true;
    rendererRef.current = renderer;
    setMounted(true);

    const handleResize = () => {
      renderer.setSize(window.innerWidth, window.innerHeight);
    };
    window.addEventListener("resize", handleResize);

    return () => {
      cancelAnimationFrame(rafRef.current);
      renderer.dispose();
      window.removeEventListener("resize", handleResize);
    };
  }, []);

  const claim = useCallback((owner: string, animate: (dt: number) => void) => {
    ownerRef.current = owner;
    animateRef.current = animate;
    clockRef.current.start();

    const loop = () => {
      const dt = clockRef.current.getDelta();
      const entry = scenesRef.current.get(owner);

      if (entry && rendererRef.current) {
        rendererRef.current.setRenderTarget(null);
        animateRef.current?.(dt);
        rendererRef.current.render(entry.scene, entry.camera);
      }
      rafRef.current = requestAnimationFrame(loop);
    };
    cancelAnimationFrame(rafRef.current);
    loop();
  }, []);

  const release = useCallback((_owner: string) => {
    cancelAnimationFrame(rafRef.current);
    ownerRef.current = null;
    animateRef.current = null;
  }, []);

  const addScene = useCallback((owner: string, scene: THREE.Scene, camera: THREE.Camera) => {
    scenesRef.current.set(owner, { scene, camera });
  }, []);

  const removeScene = useCallback((owner: string) => {
    scenesRef.current.delete(owner);
  }, []);

  return (
    <WebGLContext.Provider
      value={{
        renderer: rendererRef.current,
        claim,
        release,
        addScene,
        removeScene,
      }}
    >
      <canvas
        ref={canvasRef}
        className="fixed inset-0 z-0 pointer-events-none"
        style={{ opacity: mounted ? 1 : 0, transition: "opacity 1.5s" }}
      />
      {children}
    </WebGLContext.Provider>
  );
}
