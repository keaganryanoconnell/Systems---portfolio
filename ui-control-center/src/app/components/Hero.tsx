"use client";

import { ArrowDown, Github, Linkedin, Mail } from "lucide-react";
import dynamic from "next/dynamic";

const HeroCanvas = dynamic(() => import("./HeroCanvas"), { ssr: false });

export default function Hero() {
  return (
    <section id="hero" className="relative min-h-screen flex items-center justify-center overflow-hidden">
      {/* WebGL fallback — shown until canvas loads, hidden by canvas opacity transition */}
      <div className="absolute inset-0 bg-gradient-to-b from-[#090c10] via-[#0d1220] to-[#090c10]">
        <div className="absolute inset-0 opacity-[0.03] grid-bg" />
        <div className="absolute top-1/4 left-1/2 -translate-x-1/2 w-[700px] h-[400px] rounded-full bg-gradient-radial from-blue/8 to-transparent" />
        <div className="absolute bottom-1/3 right-1/4 w-[500px] h-[300px] rounded-full bg-gradient-radial from-gold/6 to-transparent" />
      </div>

      <HeroCanvas />

      <div className="absolute top-1/4 left-1/2 -translate-x-1/2 w-[600px] h-[300px] rounded-full bg-blue/5 blur-[120px]" />
      <div className="absolute bottom-1/4 right-1/4 w-[400px] h-[200px] rounded-full bg-gold/5 blur-[100px]" />

      <div className="relative z-10 text-center px-6 max-w-3xl">
        <div className="section-heading text-center">Principal Systems Engineer</div>

        <h1 className="text-4xl sm:text-5xl md:text-6xl lg:text-7xl font-extrabold text-text leading-[1.05] mt-4 mb-6 tracking-tight">
          Infrastructure is
          <span className="text-gold"> invisible</span>
          <br />
          until it breaks
        </h1>

        <p className="text-lg sm:text-xl text-text-soft max-w-2xl mx-auto mb-10 leading-relaxed">
          I design, build, and harden the systems that run production workloads — from
          container runtimes and distributed consensus engines to lock-free data
          structures and chaos engineering frameworks.
        </p>

        <div className="flex flex-wrap items-center justify-center gap-4 mb-16">
          <a
            href="#container"
            className="inline-flex items-center gap-2 px-6 py-3 bg-gold text-bg font-mono text-xs font-bold tracking-wider rounded-md hover:bg-gold/90 transition-colors"
          >
            View Projects <ArrowDown size={14} />
          </a>
          <div className="flex items-center gap-3">
            <a href="https://github.com/keaganryanoconnell" target="_blank" className="p-2.5 rounded-md border border-border text-text-soft hover:text-text hover:border-border-hover transition-all" title="GitHub">
              <Github size={18} />
            </a>
            <a href="https://linkedin.com/in/keaganryanoconnell" target="_blank" className="p-2.5 rounded-md border border-border text-text-soft hover:text-text hover:border-text-soft transition-all" title="LinkedIn">
              <Linkedin size={18} />
            </a>
            <a href="mailto:keaganryanoconnell@gmail.com" className="p-2.5 rounded-md border border-border text-text-soft hover:text-text hover:border-text-soft transition-all" title="Email">
              <Mail size={18} />
            </a>
          </div>
        </div>

        <div className="flex items-center justify-center gap-3 text-text-muted text-xs font-mono">
          <span className="inline-block w-1.5 h-1.5 rounded-full bg-green animate-pulse-subtle" />
          <span>Currently available for principal / staff systems engineering roles</span>
        </div>

        <div className="mt-20 animate-bounce">
          <ArrowDown size={20} className="text-text-muted mx-auto" />
        </div>
      </div>
    </section>
  );
}
