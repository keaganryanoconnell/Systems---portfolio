"use client";

import { Github, Linkedin, Mail, ArrowUp } from "lucide-react";

export function About() {
  return (
    <section id="about" className="section pb-16">
      <hr />
      <div className="section-heading mt-16">About</div>
      <h2 className="section-title">The Engineer Behind the Infrastructure</h2>

      <div className="grid md:grid-cols-2 gap-12 mt-8">
        <div className="space-y-5">
          <p className="text-text-soft leading-relaxed text-sm">
            I'm a systems engineer with deep expertise in Linux kernel internals,
            distributed systems, and performance-critical software. My work spans
            the full stack — from kernel-level isolation primitives to browser-based
            control planes.
          </p>
          <p className="text-text-soft leading-relaxed text-sm">
            I believe that great infrastructure is invisible. Users should never
            think about the container runtime, the consensus protocol, or the
            on-disk format — they should just experience reliability. Building that
            reliability requires understanding every layer from the syscall boundary
            up to the user interface.
          </p>
          <p className="text-text-soft leading-relaxed text-sm">
            Currently seeking principal or staff systems engineering roles where I can
            design and build the foundational infrastructure that teams depend on.
          </p>

          <div className="flex items-center gap-3 pt-4">
            <a href="https://github.com" target="_blank" className="flex items-center gap-2 px-4 py-2.5 bg-surface border border-border rounded-md text-xs font-mono font-bold text-text-soft hover:text-text hover:border-border-hover transition-all">
              <Github size={14} /> GitHub
            </a>
            <a href="https://linkedin.com" target="_blank" className="flex items-center gap-2 px-4 py-2.5 bg-surface border border-border rounded-md text-xs font-mono font-bold text-text-soft hover:text-text hover:border-border-hover transition-all">
              <Linkedin size={14} /> LinkedIn
            </a>
            <a href="mailto:hello@example.com" className="flex items-center gap-2 px-4 py-2.5 bg-surface border border-border rounded-md text-xs font-mono font-bold text-text-soft hover:text-text hover:border-border-hover transition-all">
              <Mail size={14} /> Email
            </a>
          </div>
        </div>

        <div className="space-y-4">
          <h4 className="text-xs font-mono font-bold text-gold tracking-wider uppercase">Expertise</h4>
          <div className="flex flex-wrap gap-1.5">
            {[
              "Rust", "Linux Kernel", "Distributed Systems", "Containers",
              "eBPF / seccomp", "Lock-free Data Structures", "TCP / Networking",
              "Consensus Protocols", "Storage Engines", "Performance Profiling",
              "Chaos Engineering", "CLI Design", "Tauri / Desktop Apps",
            ].map((t) => (
              <span key={t} className="tag">{t}</span>
            ))}
          </div>

          <h4 className="text-xs font-mono font-bold text-gold tracking-wider uppercase mt-6">Projects in this portfolio</h4>
          <ul className="space-y-2 text-xs font-mono text-text-soft">
            <li className="flex items-center gap-2">
              <span className="w-1.5 h-1.5 rounded-full bg-green" />
              <span>Container Engine — Namespace isolation, cgroups v2, seccomp BPF</span>
            </li>
            <li className="flex items-center gap-2">
              <span className="w-1.5 h-1.5 rounded-full bg-gold" />
              <span>Distributed Log Broker — Segmented append logs, lock-free SPSC buffer, binary TCP</span>
            </li>
            <li className="flex items-center gap-2">
              <span className="w-1.5 h-1.5 rounded-full bg-purple" />
              <span>Platform Nodes — Raft log consensus, SWIM Gossip membership over UDP</span>
            </li>
            <li className="flex items-center gap-2">
              <span className="w-1.5 h-1.5 rounded-full bg-blue" />
              <span>LSM Storage &amp; SQL Engine — MemTable compaction pipeline, relational query parser</span>
            </li>
            <li className="flex items-center gap-2">
              <span className="w-1.5 h-1.5 rounded-full bg-cyan" />
              <span>Custom UNIX Shell — dup2 pipe chaining, environment expansions, async job reapers</span>
            </li>
            <li className="flex items-center gap-2">
              <span className="w-1.5 h-1.5 rounded-full bg-gold" />
              <span>Bitcask KV Database — Memory-mapped log-structured files, KeyDir index, background merge</span>
            </li>
            <li className="flex items-center gap-2">
              <span className="w-1.5 h-1.5 rounded-full bg-purple" />
              <span>Async Systems Runtime — epoll Reactor event loop, work-stealing thread-pool Executor</span>
            </li>
            <li className="flex items-center gap-2">
              <span className="w-1.5 h-1.5 rounded-full bg-emerald-500" />
              <span>Control Center Workspace — Next.js, React 19, Tailwind v4, xterm.js terminal CLI</span>
            </li>
            <li className="flex items-center gap-2">
              <span className="w-1.5 h-1.5 rounded-full bg-red" />
              <span>Distributed Key-Value Store — Raft consensus, FSM replication, election timeouts, split-brain healing</span>
            </li>
          </ul>
        </div>
      </div>
    </section>
  );
}

export function Footer() {
  return (
    <footer className="border-t border-border bg-surface">
      <div className="max-w-[1200px] mx-auto px-6 py-8 flex flex-col sm:flex-row items-center justify-between gap-4">
        <div className="flex items-center gap-3 text-xs font-mono text-text-muted">
          <span className="text-gold font-bold">◈</span>
          <span>© {new Date().getFullYear()} Portfolio. Built with Next.js, Tailwind CSS, and Rust.</span>
        </div>
        <button
          onClick={() => window.scrollTo({ top: 0, behavior: "smooth" })}
          className="flex items-center gap-2 text-xs font-mono text-text-soft hover:text-gold transition-colors"
        >
          Back to top <ArrowUp size={12} />
        </button>
      </div>
    </footer>
  );
}
