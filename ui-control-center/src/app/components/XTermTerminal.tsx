"use client";

import { useEffect, useRef } from "react";

interface TerminalProps {
  onCommand?: (command: string) => void;
  chaosMode: {
    partitionSplit: boolean;
    malformedFrames: boolean;
    crashNode2: boolean;
    fuzzerRunning: boolean;
  };
}

export default function XTermTerminal({ onCommand, chaosMode }: TerminalProps) {
  const terminalRef = useRef<HTMLDivElement>(null);
  const xtermInstance = useRef<any>(null);
  const commandBuffer = useRef<string>("");

  useEffect(() => {
    let active = true;

    async function initTerminal() {
      const { Terminal } = await import("@xterm/xterm");
      await import("@xterm/xterm/css/xterm.css");

      if (!active || !terminalRef.current) return;

      // Clean up previous instance
      if (xtermInstance.current) {
        xtermInstance.current.dispose();
      }

      const term = new Terminal({
        cursorBlink: true,
        cursorStyle: "underline",
        fontSize: 12,
        fontFamily: "JetBrains Mono, Menlo, monospace",
        theme: {
          background: "#0d0221",
          foreground: "#39ff14", // Emerald
          cursor: "#00e5ff", // Cyan
          black: "#000000",
          red: "#ff2d95",
          green: "#39ff14",
          yellow: "#ffe600",
          blue: "#3b82f6",
          magenta: "#ec4899",
          cyan: "#00e5ff",
          white: "#f4f4f5",
        },
        cols: 80,
        rows: 24,
      });

      term.open(terminalRef.current);
      xtermInstance.current = term;

      // Print banner
      term.writeln("\x1b[1;36m┌────────────────────────────────────────────────────────┐\x1b[0m");
      term.writeln("\x1b[1;36m│ TAURI MULTI-PLATFORM ADMIN CONSOLE - CUSTOM POSIX TUI   │\x1b[0m");
      term.writeln("\x1b[1;36m├────────────────────────────────────────────────────────┤\x1b[0m");
      term.writeln("\x1b[1;36m│ \x1b[32mActive Core Version: 1.4.2-stable                      \x1b[1;36m│\x1b[0m");
      term.writeln("\x1b[1;36m│ \x1b[32mConsensus Protocol: Raft-SWIM Hybrid                   \x1b[1;36m│\x1b[0m");
      term.writeln("\x1b[1;36m│ \x1b[32mType 'help' to list available system commands.         \x1b[1;36m│\x1b[0m");
      term.writeln("\x1b[1;36m└────────────────────────────────────────────────────────┘\x1b[0m");
      term.writeln("");
      term.write("\x1b[1;32msys-ops@tauri-cluster\x1b[0m:\x1b[1;34m~\x1b[0m$ ");

      term.onData((data) => {
        const char = data;
        if (char === "\r") {
          // Enter key
          term.writeln("");
          const cmd = commandBuffer.current.trim();
          if (cmd) {
            handleCommand(cmd, term);
            if (onCommand) onCommand(cmd);
          } else {
            term.write("\x1b[1;32msys-ops@tauri-cluster\x1b[0m:\x1b[1;34m~\x1b[0m$ ");
          }
          commandBuffer.current = "";
        } else if (char === "\u007F") {
          // Backspace key
          if (commandBuffer.current.length > 0) {
            commandBuffer.current = commandBuffer.current.slice(0, -1);
            term.write("\b \b");
          }
        } else {
          // Normal characters
          // Ignore control escape sequences
          if (char.charCodeAt(0) >= 32 && char.charCodeAt(0) < 127) {
            commandBuffer.current += char;
            term.write(char);
          }
        }
      });
    }

    initTerminal();

    return () => {
      active = false;
      if (xtermInstance.current) {
        xtermInstance.current.dispose();
      }
    };
  }, [chaosMode]);

  const handleCommand = (cmd: string, term: any) => {
    const args = cmd.split(" ");
    const baseCmd = args[0].toLowerCase();

    switch (baseCmd) {
      case "help":
        term.writeln("\x1b[1;33mAvailable Commands:\x1b[0m");
        term.writeln("  \x1b[1;32mhelp\x1b[0m          - Display available system control actions");
        term.writeln("  \x1b[1;32mstatus\x1b[0m        - Display node health, replication & consensus state");
        term.writeln("  \x1b[1;32mnodes\x1b[0m         - Query active nodes inside the SWIM group membership");
        term.writeln("  \x1b[1;32mdb-pages\x1b[0m      - Print status of our custom B+Tree on-disk page allocation");
        term.writeln("  \x1b[1;32mcompaction\x1b[0m    - Force immediate compaction merge on LSM levels");
        term.writeln("  \x1b[1;32mchaos\x1b[0m         - List all active crash & partition simulations");
        term.writeln("  \x1b[1;32mclear\x1b[0m         - Flush local screen buffer");
        break;

      case "status":
        term.writeln("\x1b[1;36m=== TELEMETRY HEALTH REPORT ===\x1b[0m");
        term.writeln(`SWIM Membership:  \x1b[1;32mACTIVE (${chaosMode.crashNode2 ? "4/5 Nodes" : "5/5 Nodes"})\x1b[0m`);
        term.writeln(`Partition Split:  ${chaosMode.partitionSplit ? "\x1b[1;31mDETECTED (Split brain candidate nodes active)\x1b[0m" : "\x1b[1;32mNONE\x1b[0m"}`);
        term.writeln(`Fuzzing Agent:    ${chaosMode.fuzzerRunning ? "\x1b[1;33mRUNNING (High-throughput traffic load simulated)\x1b[0m" : "\x1b[1;32mIDLE\x1b[0m"}`);
        term.writeln(`Frame Integrity:  ${chaosMode.malformedFrames ? "\x1b[1;31mCOMPROMISED (Malformed frame inject filter active)\x1b[0m" : "\x1b[1;32mSECURE\x1b[0m"}`);
        term.writeln("Arena Allocators: \x1b[1;32mPool validated. Zero leaks detected.\x1b[0m");
        break;

      case "nodes":
        term.writeln("\x1b[1;36m=== SWIM CLUSTER MEMBERSHIP ===\x1b[0m");
        term.writeln("NODE_ID  ROLE       STATUS      RTT (ms)   PORT");
        term.writeln("1        LEADER     \x1b[32mHEALTHY\x1b[0m     0.15ms     4001");
        term.writeln(`2        FOLLOWER   ${chaosMode.crashNode2 ? "\x1b[31mCRASHED\x1b[0m" : "\x1b[32mHEALTHY\x1b[0m"}     ${chaosMode.crashNode2 ? "---" : "4.20ms"}     4002`);
        term.writeln(`3        FOLLOWER   ${chaosMode.partitionSplit ? "\x1b[33mUNREACH\x1b[0m" : "\x1b[32mHEALTHY\x1b[0m"}     ${chaosMode.partitionSplit ? "---" : "6.85ms"}     4003`);
        term.writeln(`4        FOLLOWER   ${chaosMode.partitionSplit ? "\x1b[33mUNREACH\x1b[0m" : "\x1b[32mHEALTHY\x1b[0m"}     ${chaosMode.partitionSplit ? "---" : "8.12ms"}     4004`);
        term.writeln(`5        FOLLOWER   ${chaosMode.partitionSplit ? "\x1b[33mUNREACH\x1b[0m" : "\x1b[32mHEALTHY\x1b[0m"}     ${chaosMode.partitionSplit ? "---" : "5.45ms"}     4005`);
        break;

      case "db-pages":
        term.writeln("\x1b[1;36m=== B+TREE PAGE INDEX SCHEMATIC ===\x1b[0m");
        term.writeln("Page ID  Type          Dirty  Key Range          Ref Count");
        term.writeln("0x0001   ROOT_NODE     No     [0000A0 - 9999Z0]  12");
        term.writeln("0x0002   INTERNAL_NODE Yes    [0000A0 - 4500T0]  2");
        term.writeln("0x0003   INTERNAL_NODE No     [4501T1 - 9999Z0]  4");
        term.writeln("0x00A1   LEAF_NODE     Yes    [0000A0 - 1200C0]  1");
        term.writeln("0x00A2   LEAF_NODE     No     [1200C1 - 3400S0]  1");
        term.writeln("0x00A3   LEAF_NODE     No     [3400S1 - 4500T0]  3");
        break;

      case "compaction":
        term.writeln("\x1b[1;33m[LSM-COMPACT] Initializing compaction filter on Levels L0 -> L1...\x1b[0m");
        term.writeln("[LSM-COMPACT] Merging 4 SSTables into Level 1 segment.");
        term.writeln("[LSM-COMPACT] Compaction completed in 24.18ms. Saved \x1b[1;32m894.2 KB\x1b[0m on disk.");
        break;

      case "chaos":
        term.writeln("\x1b[1;36m=== CHAOS INJECTION PARAMETERS ===\x1b[0m");
        term.writeln(`Network Partition: ${chaosMode.partitionSplit ? "\x1b[1;31mENABLED (Raft split brain sim)\x1b[0m" : "\x1b[1;32mDISABLED\x1b[0m"}`);
        term.writeln(`Malformed Frames:  ${chaosMode.malformedFrames ? "\x1b[1;31mENABLED (Injected noise)\x1b[0m" : "\x1b[1;32mDISABLED\x1b[0m"}`);
        term.writeln(`Crashed Node 2:    ${chaosMode.crashNode2 ? "\x1b[1;31mENABLED (SIGKILL daemon 2)\x1b[0m" : "\x1b[1;32mDISABLED\x1b[0m"}`);
        term.writeln(`Fuzzer Activity:   ${chaosMode.fuzzerRunning ? "\x1b[1;33mACTIVE (Property-based tests)\x1b[0m" : "\x1b[1;32mINACTIVE\x1b[0m"}`);
        break;

      case "clear":
        term.clear();
        break;

      default:
        term.writeln(`sys-terminal: command not found: \x1b[31m${baseCmd}\x1b[0m`);
        break;
    }

    term.write("\x1b[1;32msys-ops@tauri-cluster\x1b[0m:\x1b[1;34m~\x1b[0m$ ");
  };

  return (
    <div className="w-full h-full min-h-[160px] bg-surface rounded border border-border overflow-hidden relative flex flex-col">
      <div className="h-6 border-b border-border bg-[#090a0c] px-3 flex items-center justify-between">
        <div className="flex items-center gap-1.5">
          <span className="w-2.5 h-2.5 rounded-full bg-red"></span>
          <span className="w-2.5 h-2.5 rounded-full bg-gold"></span>
          <span className="w-2.5 h-2.5 rounded-full bg-green"></span>
          <span className="text-[10px] font-mono font-bold tracking-wider text-text-soft ml-2">ADMIN_TUI // POSIX_SHELL</span>
        </div>
        <div className="flex items-center gap-2">
          <span className="h-2 w-2 rounded-full bg-green animate-pulse"></span>
          <span className="text-[9px] font-mono text-green font-bold">TTY1_ONLINE</span>
        </div>
      </div>
      <div className="flex-1 overflow-hidden" ref={terminalRef} />
    </div>
  );
}
