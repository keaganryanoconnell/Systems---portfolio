"use client";

import { useEffect, useRef, useState, useCallback } from "react";
import { usePathname, useRouter } from "next/navigation";
import { transitionVert, transitionFrag } from "../../shaders/transitions/dissolve";

type TransitionPhase = "idle" | "capture" | "animate" | "done";

export default function ShaderTransition() {
  const pathname = usePathname();
  const router = useRouter();
  const [phase, setPhase] = useState<TransitionPhase>("idle");
  const [targetPath, setTargetPath] = useState<string | null>(null);
  const containerRef = useRef<HTMLDivElement>(null);
  const progRef = useRef(0);
  const rafRef = useRef(0);
  const startTimeRef = useRef(0);
  const prevPathRef = useRef(pathname);

  const DURATION = 600;

  useEffect(() => {
    if (pathname !== prevPathRef.current && targetPath === null) {
      // Path changed externally (browser back/forward)
      prevPathRef.current = pathname;
    }
  }, [pathname, targetPath]);

  const navigateTo = useCallback((path: string) => {
    if (phase !== "idle") return;
    setTargetPath(path);
    setPhase("capture");

    startTimeRef.current = performance.now();
    progRef.current = 0;

    const container = containerRef.current;
    if (!container) return;

    const canvas = document.createElement("canvas");
    canvas.width = window.innerWidth;
    canvas.height = window.innerHeight;
    canvas.style.position = "fixed";
    canvas.style.inset = "0";
    canvas.style.zIndex = "90";
    canvas.style.pointerEvents = "none";
    container.appendChild(canvas);

    const gl = canvas.getContext("webgl2") || canvas.getContext("webgl");
    if (!gl) return;

    const vertShader = gl.createShader(gl.VERTEX_SHADER)!;
    gl.shaderSource(vertShader, transitionVert);
    gl.compileShader(vertShader);

    const fragShader = gl.createShader(gl.FRAGMENT_SHADER)!;
    gl.shaderSource(fragShader, transitionFrag);
    gl.compileShader(fragShader);

    const program = gl.createProgram()!;
    gl.attachShader(program, vertShader);
    gl.attachShader(program, fragShader);
    gl.linkProgram(program);
    gl.useProgram(program);

    const uProgress = gl.getUniformLocation(program, "uProgress");
    const uTime = gl.getUniformLocation(program, "uTime");
    const uResolution = gl.getUniformLocation(program, "uResolution");
    gl.uniform2f(uResolution, canvas.width, canvas.height);

    const quad = new Float32Array([-1, -1, 1, -1, -1, 1, 1, 1]);
    const buf = gl.createBuffer();
    gl.bindBuffer(gl.ARRAY_BUFFER, buf);
    gl.bufferData(gl.ARRAY_BUFFER, quad, gl.STATIC_DRAW);

    const posLoc = gl.getAttribLocation(program, "position");
    gl.enableVertexAttribArray(posLoc);
    gl.vertexAttribPointer(posLoc, 2, gl.FLOAT, false, 0, 0);

    setPhase("animate");

    const animate = (now: number) => {
      const elapsed = now - startTimeRef.current;
      progRef.current = Math.min(1, elapsed / DURATION);

      gl.uniform1f(uProgress, progRef.current);
      gl.uniform1f(uTime, now * 0.001);
      gl.drawArrays(gl.TRIANGLE_STRIP, 0, 4);

      if (progRef.current < 1) {
        rafRef.current = requestAnimationFrame(animate);
      } else {
        // Transition complete
        canvas.remove();
        gl.deleteProgram(program);
        gl.deleteShader(vertShader);
        gl.deleteShader(fragShader);
        setPhase("done");
        if (targetPath) {
          router.push(targetPath);
          setTargetPath(null);
          setPhase("idle");
        }
      }
    };
    rafRef.current = requestAnimationFrame(animate);
  }, [phase, router]);

  return <div ref={containerRef} />;
}
