"use client";

import { useState } from "react";
import { MessageSquare, ThumbsUp, Reply, Search, Plus, ArrowLeft, Tag } from "lucide-react";

interface Thread {
  id: number;
  title: string;
  author: string;
  replies: number;
  likes: number;
  time: string;
  category: string;
  preview: string;
  replies_data: ReplyData[];
}

interface ReplyData {
  author: string;
  time: string;
  body: string;
  likes: number;
}

const CATEGORIES = ["All", "debugging", "performance", "security", "operations", "general"];

const THREADS: Thread[] = [
  {
    id: 1, title: "Node 2 keeps crashing under memory pressure — cgroup limits?", author: "keagan_dev", replies: 12, likes: 18, time: "2h ago", category: "debugging",
    preview: "Node 2 has crashed 3 times in the last hour. Looking at the telemetry, memory usage spikes to 512MB before the OOM killer kicks in...",
    replies_data: [
      { author: "sarah_ops", time: "1h ago", body: "Check your cgroup memory.high setting — I had the same issue with PostgreSQL containers. Try bumping it to 768MB and set memory.max to 1024MB.", likes: 8 },
      { author: "keagan_dev", time: "45m ago", body: "That fixed it! I also added memory.oom.group=0 so individual processes get killed instead of the whole cgroup. Thanks Sarah.", likes: 4 },
      { author: "mike_sec", time: "20m ago", body: "Consider also setting a seccomp profile for the container — if a process allocates too fast, seccomp BPF can catch the mmap calls before they OOM.", likes: 3 },
    ]
  },
  {
    id: 2, title: "LSM compaction tuning for high-write workloads", author: "alice_db", replies: 8, likes: 25, time: "5h ago", category: "performance",
    preview: "Running benchmarks on the LSM engine with 10K writes/sec. The default compaction strategy creates too many small SSTable files...",
    replies_data: [
      { author: "dave_perf", time: "4h ago", body: "Try tiered compaction instead of leveled. Set the tier ratio to 10x — that keeps fewer but larger SSTables. I saw a 40% reduction in read amplification with this approach.", likes: 11 },
      { author: "alice_db", time: "3h ago", body: "Great suggestion. Implemented tiered compaction and write throughput jumped from 10K to 18K ops/sec. The memtable flush interval also matters — 100ms seems optimal.", likes: 5 },
    ]
  },
  {
    id: 3, title: "Container engine seccomp profile — which syscalls to allow?", author: "mike_sec", replies: 23, likes: 42, time: "1d ago", category: "security",
    preview: "Building a production seccomp-BPF filter for the container engine. Currently allowing ~50 syscalls but want to get it down to 30...",
    replies_data: [
      { author: "jane_kernel", time: "23h ago", body: "Start with blocking: ptrace, kexec_load, reboot, swapoff, init_module, finit_module. Those are almost never needed in a container. Then look at clock_settime, settimeofday.", likes: 15 },
      { author: "mike_sec", time: "20h ago", body: "Good list. I also blocked mount, umount2, pivot_root (after initial setup), and chroot. Down to 32 syscalls now.", likes: 8 },
    ]
  },
  {
    id: 4, title: "Lock-free ring buffer — Acquire vs SeqCst ordering debate", author: "niche_concurrency", replies: 15, likes: 31, time: "2d ago", category: "performance",
    preview: "The SPSC ring buffer in core-sys uses Acquire/Release ordering. Some argue SeqCst is safer. Let's discuss...",
    replies_data: [
      { author: "rustacean99", time: "1d ago", body: "On x86_64, Acquire/Release and SeqCst generate the same machine code (MOV + MFENCE or XCHG). The difference only matters on ARM/Power where the memory model is weaker. SeqCst adds the full barrier at the cost of ~20% throughput on ARM.", likes: 12 },
    ]
  },
  {
    id: 5, title: "Tauri vs Electron for systems tooling — why I chose Tauri", author: "frontend_sys", replies: 19, likes: 36, time: "3d ago", category: "operations",
    preview: "Built the control center as a Tauri app instead of Electron. The Rust backend gives us direct access to system metrics...",
    replies_data: [
      { author: "web_eng", time: "2d ago", body: "The binary size difference is insane — our Tauri build is 8MB vs Electron's 120MB+. Plus the IPC is faster since it goes through Rust channels instead of Node.js.", likes: 14 },
    ]
  },
  {
    id: 6, title: "SWIM gossip convergence time in 5-node cluster", author: "dist_sys", replies: 7, likes: 14, time: "4d ago", category: "operations",
    preview: "Testing convergence time with the SWIM protocol. With 5 nodes, failure detection takes ~500ms on average...",
    replies_data: [
      { author: "raft_fan", time: "3d ago", body: "500ms is solid. Are you using the suspicion mechanism or direct failure detection? Suspicion adds a round of gossip but reduces false positives from transient network issues.", likes: 6 },
    ]
  },
];

export default function Forum() {
  const [view, setView] = useState<"list" | "thread">("list");
  const [selectedThread, setSelectedThread] = useState<Thread | null>(null);
  const [activeCategory, setActiveCategory] = useState("All");
  const [search, setSearch] = useState("");
  const [replyText, setReplyText] = useState("");

  const filtered = THREADS.filter((t) => {
    if (activeCategory !== "All" && t.category !== activeCategory) return false;
    if (search && !t.title.toLowerCase().includes(search.toLowerCase())) return false;
    return true;
  });

  if (view === "thread" && selectedThread) {
    return (
      <section id="forum" className="section">
        <button
          onClick={() => { setView("list"); setSelectedThread(null); }}
          className="flex items-center gap-2 text-xs font-mono font-bold text-text-soft hover:text-text transition-colors mb-6"
        >
          <ArrowLeft size={14} /> Back to discussions
        </button>

        <div className="cyber-panel p-6 mb-6">
          <div className="flex items-center gap-2 mb-3">
            <span className="text-[10px] font-mono uppercase tracking-wider text-gold px-2 py-0.5 rounded bg-gold-bg border border-gold-border">
              #{selectedThread.category}
            </span>
            <span className="text-xs text-text-muted font-mono">
              Started by {selectedThread.author} · {selectedThread.time}
            </span>
          </div>
          <h3 className="text-xl font-bold text-text mb-3">{selectedThread.title}</h3>
          <p className="text-sm text-text-soft leading-relaxed">{selectedThread.preview}</p>
          <div className="flex items-center gap-4 mt-4">
            <button className="flex items-center gap-1.5 text-xs font-mono text-text-muted hover:text-gold transition-colors">
              <ThumbsUp size={13} /> {selectedThread.likes}
            </button>
          </div>
        </div>

        {selectedThread.replies_data.map((r, i) => (
          <div key={i} className="cyber-panel p-5 mb-3 ml-6 border-l-2 border-gold/20">
            <div className="flex items-center gap-2 mb-2">
              <span className="text-xs font-mono font-bold text-text">{r.author}</span>
              <span className="text-[10px] font-mono text-text-muted">{r.time}</span>
            </div>
            <p className="text-sm text-text-soft leading-relaxed mb-3">{r.body}</p>
            <button className="flex items-center gap-1.5 text-xs font-mono text-text-muted hover:text-gold transition-colors">
              <ThumbsUp size={12} /> {r.likes}
            </button>
          </div>
        ))}

        <div className="cyber-panel p-4 mt-6">
          <textarea
            value={replyText}
            onChange={(e) => setReplyText(e.target.value)}
            placeholder="Write a reply..."
            className="w-full bg-bg border border-border rounded-md p-3 text-sm text-text placeholder-text-muted font-sans resize-none focus:outline-none focus:border-blue-border mb-3"
            rows={3}
          />
          <button
            onClick={() => {
              if (replyText.trim()) {
                setReplyText("");
              }
            }}
            className="px-4 py-2 bg-blue text-white text-xs font-mono font-bold rounded-md hover:bg-blue/90 transition-colors"
          >
            Post Reply
          </button>
        </div>
      </section>
    );
  }

  return (
    <section id="forum" className="section">
      <div className="section-heading">Community</div>
      <h2 className="section-title">Engineering Discussions</h2>
      <p className="text-text-soft text-base max-w-2xl mb-8">
        Technical deep-dives, debugging war stories, and architecture discussions
        from engineers working on similar systems problems.
      </p>

      <div className="flex flex-wrap items-center gap-3 mb-6">
        <div className="relative flex-1 max-w-sm">
          <Search size={14} className="absolute left-3 top-1/2 -translate-y-1/2 text-text-muted" />
          <input
            value={search}
            onChange={(e) => setSearch(e.target.value)}
            placeholder="Search discussions..."
            className="w-full pl-9 pr-3 py-2 bg-bg border border-border rounded-md text-xs font-mono text-text placeholder-text-muted focus:outline-none focus:border-blue-border"
          />
        </div>
        {CATEGORIES.map((cat) => (
          <button
            key={cat}
            onClick={() => setActiveCategory(cat)}
            className={`px-3 py-1.5 rounded text-[10px] font-mono font-bold uppercase tracking-wider transition-all ${
              activeCategory === cat
                ? "bg-gold-bg text-gold border border-gold-border"
                : "text-text-muted border border-border hover:text-text"
            }`}
          >
            {cat}
          </button>
        ))}
      </div>

      <div className="space-y-3">
        {filtered.map((thread) => (
          <div
            key={thread.id}
            onClick={() => { setSelectedThread(thread); setView("thread"); }}
            className="cyber-panel p-5 cursor-pointer hover:border-border-hover transition-all group"
          >
            <div className="flex items-start justify-between gap-4">
              <div className="flex-1 min-w-0">
                <div className="flex items-center gap-2 mb-1.5">
                  <span className="text-[10px] font-mono uppercase tracking-wider text-gold px-2 py-0.5 rounded bg-gold-bg border border-gold-border">
                    #{thread.category}
                  </span>
                  <span className="w-1.5 h-1.5 rounded-full bg-green" />
                </div>
                <h4 className="text-sm font-bold text-text group-hover:text-blue transition-colors mb-1">
                  {thread.title}
                </h4>
                <p className="text-xs text-text-soft line-clamp-1">{thread.preview}</p>
              </div>
              <div className="flex items-center gap-4 shrink-0 text-xs font-mono text-text-muted">
                <span className="flex items-center gap-1"><MessageSquare size={12} /> {thread.replies}</span>
                <span className="flex items-center gap-1"><ThumbsUp size={12} /> {thread.likes}</span>
                <span>{thread.time}</span>
              </div>
            </div>
          </div>
        ))}
      </div>

      <div className="mt-6 text-center">
        <button className="inline-flex items-center gap-2 px-4 py-2 border border-border rounded-md text-xs font-mono font-bold text-text-soft hover:text-text hover:border-border-hover transition-all">
          <Plus size={14} /> New Discussion
        </button>
      </div>
    </section>
  );
}
