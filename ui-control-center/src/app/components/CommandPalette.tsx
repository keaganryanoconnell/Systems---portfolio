"use client";

import { useEffect, useState, useCallback } from "react";
import { Command } from "cmdk";
import { useRouter } from "next/navigation";
import {
  Home, Network, Container, BookOpen, MessageSquare, User,
  Zap, ExternalLink, FileDown, Github, Linkedin, Mail, Volume2, VolumeX,
} from "lucide-react";
import { useUISounds, toggleMute, isMuted } from "../hooks/useUISounds";

interface PaletteItem {
  id: string;
  label: string;
  icon: React.ComponentType<{ size?: number }>;
  section: string;
  shortcut?: string;
  action: () => void;
}

export default function CommandPalette() {
  const [open, setOpen] = useState(false);
  const router = useRouter();
  const { play } = useUISounds();
  const [soundMuted, setSoundMuted] = useState(true);

  const items: PaletteItem[] = [
    { id: "hero", label: "Go Home", icon: Home, section: "Navigation", shortcut: "H", action: () => scrollTo("hero") },
    { id: "arch", label: "Architecture", icon: Network, section: "Navigation", shortcut: "A", action: () => scrollTo("arch") },
    { id: "workspace", label: "Workspace", icon: Container, section: "Navigation", shortcut: "W", action: () => scrollTo("workspace") },
    { id: "deepdives", label: "Deep Dives", icon: BookOpen, section: "Navigation", shortcut: "D", action: () => scrollTo("deepdives") },
    { id: "forum", label: "Forum", icon: MessageSquare, section: "Navigation", shortcut: "F", action: () => scrollTo("forum") },
    { id: "about", label: "About", icon: User, section: "Navigation", shortcut: "B", action: () => scrollTo("about") },
    { id: "capstone", label: "Open Capstone Console", icon: Zap, section: "Capstone", shortcut: "C", action: () => router.push("/capstone") },
    { id: "resume", label: "Download Resume", icon: FileDown, section: "Actions", action: () => window.open("/resume.pdf") },
    {
      id: "sound", label: soundMuted ? "Enable Sound" : "Disable Sound", icon: soundMuted ? VolumeX : Volume2,
      section: "Actions", action: () => { const m = toggleMute(); setSoundMuted(m); play("click"); }
    },
    { id: "source", label: "View Source Code", icon: ExternalLink, section: "Links", action: () => window.open("https://github.com/keaganryanoconnell/Systems---portfolio", "_blank") },
    { id: "github", label: "GitHub Profile", icon: Github, section: "Links", action: () => window.open("https://github.com/keaganryanoconnell", "_blank") },
    { id: "linkedin", label: "LinkedIn Profile", icon: Linkedin, section: "Links", action: () => window.open("https://linkedin.com/in/keaganryanoconnell", "_blank") },
    { id: "email", label: "Send Email", icon: Mail, section: "Links", action: () => window.location.href = "mailto:keagan@ryanonnell.com" },
  ];

  const scrollTo = (id: string) => {
    setOpen(false);
    setTimeout(() => {
      document.getElementById(id)?.scrollIntoView({ behavior: "smooth" });
    }, 100);
  };

  const down = useCallback((e: KeyboardEvent) => {
    if (e.key === "k" && (e.metaKey || e.ctrlKey)) {
      e.preventDefault();
      setOpen((v) => !v);
      if (!open) play("pop");
    }
    if (e.key === "Escape" && open) {
      setOpen(false);
      play("whoosh");
    }
  }, [open, play]);

  useEffect(() => {
    document.addEventListener("keydown", down);
    return () => document.removeEventListener("keydown", down);
  }, [down]);

  return (
    <Command.Dialog
      open={open}
      onOpenChange={setOpen}
      label="Command Palette"
      className="fixed inset-0 z-[100]"
    >
      <div className="fixed inset-0 bg-black/60 backdrop-blur-xl" onClick={() => setOpen(false)} />
      <div className="fixed inset-0 flex items-start justify-center pt-[20vh]">
        <div className="w-full max-w-[560px] bg-[#13161d] border border-[#252a36] rounded-xl overflow-hidden shadow-2xl">
          <Command.Input
            placeholder="Type a command or search..."
            className="w-full bg-transparent px-5 py-4 text-sm font-mono text-[#e1e4ea] placeholder-[#5c6270] border-b border-[#252a36] outline-none"
            autoFocus
          />
          <Command.List className="max-h-[320px] overflow-y-auto p-2 scrollbar-thin">
            <Command.Empty className="py-8 text-center text-xs font-mono text-[#5c6270]">
              No results found.
            </Command.Empty>
            {["Navigation", "Capstone", "Actions", "Links"].map((section) => (
              <Command.Group key={section} heading={section} className="[&_[cmdk-group-heading]]:text-[9px] [&_[cmdk-group-heading]]:font-mono [&_[cmdk-group-heading]]:font-bold [&_[cmdk-group-heading]]:text-[#d2991d] [&_[cmdk-group-heading]]:uppercase [&_[cmdk-group-heading]]:tracking-wider [&_[cmdk-group-heading]]:px-3 [&_[cmdk-group-heading]]:py-2">
                {items.filter((i) => i.section === section).map((item) => (
                  <Command.Item
                    key={item.id}
                    value={item.label}
                    onSelect={() => { item.action(); play("click"); }}
                    className="flex items-center gap-3 px-3 py-2.5 rounded-lg text-xs font-mono text-[#e1e4ea] cursor-pointer data-[selected=true]:bg-[#d2991d]/10 data-[selected=true]:text-[#d2991d] transition-colors"
                  >
                    <item.icon size={14} />
                    <span className="flex-1">{item.label}</span>
                    {item.shortcut && (
                      <kbd className="text-[9px] text-[#5c6270] bg-[#1a1e27] px-1.5 py-0.5 rounded border border-[#252a36]">
                        {item.shortcut}
                      </kbd>
                    )}
                  </Command.Item>
                ))}
              </Command.Group>
            ))}
          </Command.List>
          <div className="border-t border-[#252a36] px-4 py-2 flex items-center justify-between text-[9px] font-mono text-[#5c6270]">
            <span>
              <kbd className="bg-[#1a1e27] px-1 py-0.5 rounded border border-[#252a36] mr-1">↑↓</kbd>
              Navigate
            </span>
            <span>
              <kbd className="bg-[#1a1e27] px-1 py-0.5 rounded border border-[#252a36] mr-1">↵</kbd>
              Select
            </span>
            <span>
              <kbd className="bg-[#1a1e27] px-1 py-0.5 rounded border border-[#252a36] mr-1">Esc</kbd>
              Close
            </span>
          </div>
        </div>
      </div>
    </Command.Dialog>
  );
}
