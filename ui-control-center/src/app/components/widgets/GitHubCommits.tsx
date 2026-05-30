"use client";

import { useEffect, useState, useRef } from "react";
import { GitBranch } from "lucide-react";

interface CommitEvent {
  repo: string;
  branch: string;
  message: string;
  time: string;
}

export default function GitHubCommits() {
  const [events, setEvents] = useState<CommitEvent[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState(false);
  const typedRef = useRef<HTMLDivElement>(null);
  const indexRef = useRef(0);

  useEffect(() => {
    fetch("https://api.github.com/users/keaganryanoconnell/events/public")
      .then((r) => r.json())
      .then((data: any[]) => {
        const commits: CommitEvent[] = [];
        for (const event of data) {
          if (commits.length >= 5) break;
          if (event.type === "PushEvent") {
            commits.push({
              repo: event.repo.name.split("/")[1] || event.repo.name,
              branch: (event.payload.ref || "").replace("refs/heads/", ""),
              message: `${event.payload.commits?.length || 0} commits`,
              time: new Date(event.created_at).toLocaleDateString(),
            });
          }
        }
        setEvents(commits);
        setLoading(false);
      })
      .catch(() => {
        setError(true);
        setLoading(false);
      });
  }, []);

  useEffect(() => {
    if (events.length === 0 || !typedRef.current) return;
    const el = typedRef.current;
    const interval = setInterval(() => {
      indexRef.current++;
      if (indexRef.current > events.length * 3) indexRef.current = 0;
      const lines = events.map((e) => {
        const idx = events.indexOf(e);
        const typed = indexRef.current >= idx * 3;
        if (typed) {
          return `> git push origin ${e.branch} — ${e.message}  [${e.repo}] ${e.time}`;
        }
        return "";
      });
      el.innerHTML = lines.filter(Boolean).join("<br/>");
    }, 1200);
    return () => clearInterval(interval);
  }, [events]);

  return (
    <div className="cyber-panel p-4 border-l-4 border-l-[#3fb950]">
      <div className="flex items-center gap-2 mb-3 pb-2 border-b border-border">
        <GitBranch size={14} className="text-[#3fb950]" />
        <span className="text-[9px] font-mono font-bold text-[#3fb950] tracking-wider uppercase">
          GitHub Commits
        </span>
      </div>
      {loading ? (
        <div className="text-[10px] font-mono text-text-muted animate-pulse">Loading commits...</div>
      ) : error ? (
        <div className="text-[10px] font-mono text-red">API rate limit. Try again later.</div>
      ) : (
        <div
          ref={typedRef}
          className="text-[10px] font-mono text-text-soft leading-relaxed min-h-[60px]"
        />
      )}
    </div>
  );
}
