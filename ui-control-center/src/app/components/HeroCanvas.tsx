"use client";

import { useEffect, useRef } from "react";
import * as THREE from "three";
import { heroSceneVert, heroSceneFrag } from "../../shaders/hero_scene";

export default function HeroCanvas() {
  const containerRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const container = containerRef.current;
    if (!container) return;

    const w = window.innerWidth;
    const h = window.innerHeight;

    const renderer = new THREE.WebGLRenderer({ alpha: true, antialias: true });
    renderer.setPixelRatio(Math.min(window.devicePixelRatio, 2));
    renderer.setSize(w, h);
    container.appendChild(renderer.domElement);
    renderer.domElement.style.position = "absolute";
    renderer.domElement.style.inset = "0";
    renderer.domElement.style.pointerEvents = "none";
    renderer.domElement.style.zIndex = "0";

    const scene = new THREE.Scene();
    const camera = new THREE.OrthographicCamera(-1, 1, 1, -1, 0.1, 10);
    camera.position.z = 5;

    const uniforms = {
      uTime: { value: 0 },
      uMouse: { value: new THREE.Vector2(0, 0) },
      uScroll: { value: 0 },
      uResolution: { value: new THREE.Vector2(w, h) },
    };

    const material = new THREE.ShaderMaterial({
      vertexShader: heroSceneVert,
      fragmentShader: heroSceneFrag,
      uniforms,
      transparent: true,
    });

    const geometry = new THREE.PlaneGeometry(2, 2);
    const mesh = new THREE.Mesh(geometry, material);
    scene.add(mesh);

    let visible = true;
    let scrollVal = 0;
    let mouseX = 0;
    let mouseY = 0;
    let animId = 0;
    const clock = new THREE.Clock();

    const observer = new IntersectionObserver(
      ([entry]) => {
        visible = entry.isIntersecting;
      },
      { threshold: 0.1 }
    );
    observer.observe(container);

    const onMouse = (e: MouseEvent) => {
      mouseX = (e.clientX / w) * 2 - 1;
      mouseY = -(e.clientY / h) * 2 + 1;
    };
    const onScroll = () => {
      scrollVal = window.scrollY / (document.body.scrollHeight - h);
    };
    const onResize = () => {
      const nw = window.innerWidth;
      const nh = window.innerHeight;
      renderer.setSize(nw, nh);
      uniforms.uResolution.value.set(nw, nh);
    };

    window.addEventListener("mousemove", onMouse, { passive: true });
    window.addEventListener("scroll", onScroll, { passive: true });
    window.addEventListener("resize", onResize);

    const animate = () => {
      animId = requestAnimationFrame(animate);
      if (!visible) return;
      uniforms.uTime.value = clock.getElapsedTime();
      uniforms.uMouse.value.set(mouseX, mouseY);
      uniforms.uScroll.value = scrollVal;
      renderer.render(scene, camera);
    };
    animate();

    return () => {
      cancelAnimationFrame(animId);
      observer.disconnect();
      window.removeEventListener("mousemove", onMouse);
      window.removeEventListener("scroll", onScroll);
      window.removeEventListener("resize", onResize);
      renderer.dispose();
      material.dispose();
      geometry.dispose();
    };
  }, []);

  return (
    <div
      ref={containerRef}
      className="absolute inset-0 z-0 animate-fade-in opacity-0"
      style={{ animationDelay: "0.3s", animationFillMode: "forwards" }}
    />
  );
}
