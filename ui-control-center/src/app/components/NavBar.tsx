"use client";

import { useState, useEffect } from "react";
import { Home, Network, Container, MessageSquare, BookOpen, User } from "lucide-react";

const SECTIONS = [
  { id: "hero",      label: "Home",         icon: Home },
  { id: "arch",      label: "Architecture", icon: Network },
  { id: "workspace", label: "Workspace",    icon: Container },
  { id: "deepdives", label: "Deep Dives",   icon: BookOpen },
  { id: "forum",     label: "Forum",        icon: MessageSquare },
  { id: "about",     label: "About",        icon: User },
];

export default function NavBar() {
  const [active, setActive] = useState("hero");
  const [scrolled, setScrolled] = useState(false);
  const [mobileOpen, setMobileOpen] = useState(false);

  useEffect(() => {
    const onScroll = () => {
      setScrolled(window.scrollY > 60);

      const scrollPos = window.scrollY + 120;
      for (let i = SECTIONS.length - 1; i >= 0; i--) {
        const el = document.getElementById(SECTIONS[i].id);
        if (el && el.offsetTop <= scrollPos) {
          setActive(SECTIONS[i].id);
          break;
        }
      }
    };
    window.addEventListener("scroll", onScroll, { passive: true });
    return () => window.removeEventListener("scroll", onScroll);
  }, []);

  const scrollTo = (id: string) => {
    setMobileOpen(false);
    const el = document.getElementById(id);
    if (el) el.scrollIntoView({ behavior: "smooth" });
  };

  return (
    <nav
      className={`fixed top-0 left-0 right-0 z-50 transition-all duration-300 ${
        scrolled
          ? "bg-surface/95 backdrop-blur-md border-b border-border shadow-lg shadow-black/20"
          : "bg-transparent"
      }`}
    >
      <div className="max-w-[1200px] mx-auto flex items-center justify-between h-16 px-6">
        <button onClick={() => scrollTo("hero")} className="flex items-center gap-2 group">
          <span className="text-gold font-mono text-lg font-bold group-hover:drop-shadow-[0_0_8px_rgba(210,153,29,0.4)] transition-all">◈</span>
          <span className="text-text font-mono text-xs font-bold tracking-wider hidden sm:inline">PORTFOLIO</span>
        </button>

        <div className="hidden lg:flex items-center gap-1">
          {SECTIONS.map(({ id, label }) => (
            <button
              key={id}
              onClick={() => scrollTo(id)}
              className={`px-3 py-1.5 text-[11px] font-mono font-semibold tracking-wider rounded transition-all ${
                active === id
                  ? "text-gold bg-gold-bg"
                  : "text-text-soft hover:text-text hover:bg-surface"
              }`}
            >
              {label.toUpperCase()}
            </button>
          ))}
        </div>

        <button
          className="lg:hidden text-text-soft hover:text-text transition-colors"
          onClick={() => setMobileOpen(!mobileOpen)}
        >
          {mobileOpen ? "✕" : "☰"}
        </button>
      </div>

      {mobileOpen && (
        <div className="lg:hidden bg-surface border-b border-border px-4 py-3 flex flex-col gap-1">
          {SECTIONS.map(({ id, label, icon: Icon }) => (
            <button
              key={id}
              onClick={() => scrollTo(id)}
              className={`flex items-center gap-3 px-3 py-2.5 rounded text-xs font-mono font-semibold tracking-wider transition-all ${
                active === id
                  ? "text-gold bg-gold-bg"
                  : "text-text-soft hover:text-text"
              }`}
            >
              <Icon size={14} />
              {label.toUpperCase()}
            </button>
          ))}
        </div>
      )}
    </nav>
  );
}
