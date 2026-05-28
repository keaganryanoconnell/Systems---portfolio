"use client";

import { useState, useRef, useEffect } from "react";

interface ViewSqlAnalyzerProps {
  chaosMode: { partitionSplit: boolean; malformedFrames: boolean; crashNode2: boolean; fuzzerRunning: boolean; };
}

type ConsoleMode = 'kv' | 'sql';

interface DbPage {
  id: string;
  type: 'ROOT_NODE' | 'INTERNAL_NODE' | 'LEAF_NODE' | 'MEMTABLE' | 'SSTABLE';
  dirty: boolean;
  keyRange: string;
  refCount: number;
  level?: number;
}

export default function ViewSqlAnalyzer({ chaosMode }: ViewSqlAnalyzerProps) {
  const [mode, setMode] = useState<ConsoleMode>('kv');
  const [kvInput, setKvInput] = useState('');
  const [sqlInput, setSqlInput] = useState('');
  const [output, setOutput] = useState<string[]>([]);
  const [activePages, setActivePages] = useState<DbPage[]>([]);
  const [queryRunning, setQueryRunning] = useState(false);
  const outputRef = useRef<HTMLDivElement>(null);
  const dbCanvasRef = useRef<HTMLCanvasElement>(null);
  const dbContainerRef = useRef<HTMLDivElement>(null);
  const [dbDimensions, setDbDimensions] = useState({ width: 600, height: 280 });

  // Sample DB pages for visualization
  const samplePages: DbPage[] = [
    { id: '0x0001', type: 'ROOT_NODE', dirty: false, keyRange: '[0000A0 - 9999Z0]', refCount: 12 },
    { id: '0x0002', type: 'INTERNAL_NODE', dirty: true, keyRange: '[0000A0 - 4500T0]', refCount: 2 },
    { id: '0x0003', type: 'INTERNAL_NODE', dirty: false, keyRange: '[4501T1 - 9999Z0]', refCount: 4 },
    { id: '0x00A1', type: 'LEAF_NODE', dirty: true, keyRange: '[0000A0 - 1200C0]', refCount: 1 },
    { id: '0x00A2', type: 'LEAF_NODE', dirty: false, keyRange: '[1200C1 - 3400S0]', refCount: 1 },
    { id: '0x00A3', type: 'LEAF_NODE', dirty: false, keyRange: '[3400S1 - 4500T0]', refCount: 3 },
    { id: '0x00B1', type: 'LEAF_NODE', dirty: true, keyRange: '[4501T1 - 6000A0]', refCount: 1 },
    { id: '0x00B2', type: 'LEAF_NODE', dirty: false, keyRange: '[6000A1 - 8100K0]', refCount: 2 },
    { id: '0x00B3', type: 'LEAF_NODE', dirty: false, keyRange: '[8100K1 - 9999Z0]', refCount: 1 },
    { id: '0x1001', type: 'MEMTABLE', dirty: true, keyRange: 'In-Memory Buffer', refCount: 0, level: 0 },
    { id: '0x2001', type: 'SSTABLE', dirty: false, keyRange: '[Flushed L0]', refCount: 0, level: 1 },
    { id: '0x2002', type: 'SSTABLE', dirty: false, keyRange: '[Compacted L1]', refCount: 0, level: 2 },
  ];

  // Handle resize of DB visualizer
  useEffect(() => {
    if (!dbContainerRef.current) return;
    const observer = new ResizeObserver((entries) => {
      for (let entry of entries) {
        setDbDimensions({
          width: Math.floor(entry.contentRect.width),
          height: Math.floor(entry.contentRect.height),
        });
      }
    });
    observer.observe(dbContainerRef.current);
    return () => observer.disconnect();
  }, []);

  // Autoscroll output
  useEffect(() => {
    if (outputRef.current) {
      outputRef.current.scrollTop = outputRef.current.scrollHeight;
    }
  }, [output]);

  // Simulate DB page access animation - highlight recently accessed pages
  const simulateQuery = (queryText: string) => {
    setQueryRunning(true);
    setOutput(prev => [...prev, `\x1b[36m[SESSION]\x1b[0m > ${queryText}`]);

    if (mode === 'kv') {
      const parts = queryText.trim().split(/\s+/);
      const cmd = parts[0]?.toUpperCase();

      setTimeout(() => {
        if (cmd === 'PUT') {
          const key = parts[1] || 'unknown';
          const val = parts.slice(2).join(' ') || 'null';
          setOutput(prev => [...prev, `\x1b[32m[STORED]\x1b[0m key="${key}" value="${val}" → LSM MemTable (L0)`]);
          setActivePages(prev => [...prev.slice(0, 6), {
            id: `0x${Math.floor(Math.random() * 0xFFFF).toString(16).toUpperCase().padStart(4, '0')}`,
            type: 'LEAF_NODE', dirty: true, keyRange: `[${key}]`, refCount: 1
          }]);
        } else if (cmd === 'GET') {
          const key = parts[1] || 'unknown';
          setOutput(prev => [...prev, `\x1b[33m[FETCHED]\x1b[0m key="${key}" → B+Tree leaf scan (page 0x00A1)`]);
        } else if (cmd === 'DELETE') {
          const key = parts[1] || 'unknown';
          setOutput(prev => [...prev, `\x1b[31m[TOMBSTONE]\x1b[0m key="${key}" → compaction pending`]);
        } else {
          setOutput(prev => [...prev, `\x1b[31m[ERROR]\x1b[0m Unknown KV operation: ${cmd}. Use PUT, GET, DELETE.`]);
        }
        setQueryRunning(false);
      }, 200 + Math.random() * 300);
    } else {
      // SQL mode
      const isSelect = queryText.trim().toUpperCase().startsWith('SELECT');
      const isInsert = queryText.trim().toUpperCase().startsWith('INSERT');
      const isCreate = queryText.trim().toUpperCase().startsWith('CREATE');

      setTimeout(() => {
        if (isSelect) {
          setOutput(prev => [...prev,
            `\x1b[36m[PARSER]\x1b[0m SQL AST parsed (B+Tree execution path)`,
            `\x1b[36m[EXEC]\x1b[0m → Scanning index: B+Tree root (page 0x0001) → internal pages scanned (2)`,
            `\x1b[36m[EXEC]\x1b[0m → Leaf page fetch: 0x00A1 - 0x00B3 (3 pages accessed)` +
            (chaosMode.partitionSplit ? ` \x1b[31m[PARTITION BLOCK]\x1b[0m` : ``),
            `\x1b[32m[RESULT]\x1b[0m 142 rows returned in 2.34ms (${chaosMode.partitionSplit ? 'PARTIALLY_AVAILABLE' : 'FULL CONSISTENCY'})`,
          ]);
        } else if (isInsert) {
          setOutput(prev => [...prev,
            `\x1b[33m[WRITE]\x1b[0m Insert into MemTable (L0) → tx log flushed`,
            `\x1b[32m[COMMIT]\x1b[0m WAL fsync: 0.18ms. Quorum ACK: ${chaosMode.crashNode2 ? '3/4' : '4/5'} nodes confirmed.`,
          ]);
        } else if (isCreate) {
          setOutput(prev => [...prev,
            `\x1b[35m[DDL]\x1b[0m CREATE TABLE schema recorded in system catalog`,
            `\x1b[32m[OK]\x1b[0m Table created. 0 rows affected.`,
          ]);
        } else {
          setOutput(prev => [...prev, `\x1b[31m[PARSE ERROR]\x1b[0m at line 1, col 1: expected SELECT/INSERT/CREATE`]);
        }
        setQueryRunning(false);
      }, 150 + Math.random() * 400);
    }
  };

  const handleExecute = () => {
    const text = mode === 'kv' ? kvInput.trim() : sqlInput.trim();
    if (!text) return;
    simulateQuery(text);
    if (mode === 'kv') setKvInput('');
  };

  // DB Page Visualizer Canvas
  useEffect(() => {
    const canvas = dbCanvasRef.current;
    if (!canvas) return;

    const ctx = canvas.getContext("2d");
    if (!ctx) return;

    const { width, height } = dbDimensions;
    ctx.clearRect(0, 0, width, height);

    // Background grid
    ctx.strokeStyle = "rgba(255,255,255,0.02)";
    ctx.lineWidth = 1;
    for (let x = 0; x < width; x += 40) {
      ctx.beginPath(); ctx.moveTo(x, 0); ctx.lineTo(x, height); ctx.stroke();
    }
    for (let y = 0; y < height; y += 40) {
      ctx.beginPath(); ctx.moveTo(0, y); ctx.lineTo(width, y); ctx.stroke();
    }

    // Draw LSM Tree levels as columns
    // Level 0: MemTable (left), Level 1: SSTable L0, Level 2: SSTable L1
    const colWidth = 80;
    const colGap = 20;
    const startX = 20;

    const levels = [
      { label: "MEMTABLE L0", y: 20, pageCount: 2, color: "#00e5ff", baseX: startX },
      { label: "SSTABLE L1", y: 20, pageCount: 3, color: "#39ff14", baseX: startX + colWidth + colGap },
      { label: "SSTABLE L2", y: 20, pageCount: 2, color: "#ffe600", baseX: startX + (colWidth + colGap) * 2 },
    ];

    // Draw column backgrounds
    levels.forEach((level) => {
      const colH = height - 80;
      ctx.fillStyle = "rgba(20, 8, 50, 0.8)";
      ctx.strokeStyle = "rgba(26, 28, 35, 0.5)";
      ctx.lineWidth = 1;
      ctx.beginPath();
      ctx.roundRect?.(level.baseX - 5, level.y - 5, colWidth + 10, colH + 10, 4);
      ctx.fill();
      ctx.stroke();

      // Column label
      ctx.fillStyle = "#a0a0b0";
      ctx.font = "8px var(--font-mono)";
      ctx.textAlign = "center";
      ctx.fillText(level.label, level.baseX + colWidth / 2, level.y + colH + 20);
    });

    // Draw pages as boxes within each column
    const allPages = [...samplePages];
    const pagesByLevel: Record<number, DbPage[]> = { 0: [], 1: [], 2: [] };
    allPages.forEach(p => {
      if (p.type === 'MEMTABLE') pagesByLevel[0].push(p);
      else if (p.type === 'SSTABLE') pagesByLevel[p.level || 1].push(p);
      else return;
    });

    // For B+Tree pages, draw them in a tree connected layout
    const treePages = allPages.filter(p => p.type !== 'MEMTABLE' && p.type !== 'SSTABLE');
    
    // Draw connected B+Tree pages
    const treeStartX = startX + (colWidth + colGap) * 3 + 30;
    const treeWidth = width - treeStartX - 20;
    
    if (treeWidth > 100) {
      ctx.fillStyle = "rgba(20, 8, 50, 0.8)";
      ctx.strokeStyle = "rgba(26, 28, 35, 0.5)";
      ctx.lineWidth = 1;
      ctx.beginPath();
      ctx.roundRect?.(treeStartX - 5, 15, treeWidth + 10, height - 95, 4);
      ctx.fill();
      ctx.stroke();

      ctx.fillStyle = "#a0a0b0";
      ctx.font = "8px var(--font-mono)";
      ctx.textAlign = "center";
      ctx.fillText("B+TREE PAGE INDEX", treeStartX + treeWidth / 2, height - 60);

      // Root node at top center
      const root = treePages.find(p => p.type === 'ROOT_NODE');
      if (root) {
        const rx = treeStartX + treeWidth / 2;
        const ry = 40;
        drawPageBox(ctx, rx, ry, root, true);
      }

      // Internal nodes
      const internalNodes = treePages.filter(p => p.type === 'INTERNAL_NODE');
      internalNodes.forEach((node, i) => {
        const nx = treeStartX + treeWidth * (i + 1) / (internalNodes.length + 1);
        const ny = 95;
        drawPageBox(ctx, nx, ny, node, false);

        // Connect to root
        ctx.beginPath();
        ctx.moveTo(treeStartX + treeWidth / 2, 55);
        ctx.lineTo(nx, ny - 5);
        ctx.strokeStyle = "rgba(59, 130, 246, 0.15)";
        ctx.lineWidth = 1;
        ctx.stroke();
      });

      // Leaf nodes at bottom
      const leafNodes = treePages.filter(p => p.type === 'LEAF_NODE');
      leafNodes.forEach((node, i) => {
        // Pack more leaves
        const leavesPerRow = Math.min(5, leafNodes.length);
        const rowIdx = Math.floor(i / leavesPerRow);
        const colInRow = i % leavesPerRow;
        const lx = treeStartX + 10 + colInRow * (Math.min(70, (treeWidth - 20) / leavesPerRow));
        const ly = 145 + rowIdx * 50;
        
        if (lx + 60 < treeStartX + treeWidth) {
          drawPageBox(ctx, lx, ly, node, false, Math.min(60, (treeWidth - 20) / leavesPerRow) - 4);
          
          // Connect to parent internal node
          const parentIdx = node.id.startsWith('0x00A') ? 0 : node.id.startsWith('0x00B') ? 1 : 0;
          const parentNode = internalNodes[parentIdx] || internalNodes[0];
          const parentX = treeStartX + treeWidth * (internalNodes.indexOf(parentNode) + 1) / (internalNodes.length + 1);
          
          ctx.beginPath();
          ctx.moveTo(lx + 25, ly - 3);
          ctx.lineTo(parentX, 105);
          ctx.strokeStyle = "rgba(57, 255, 20, 0.08)";
          ctx.lineWidth = 1;
          ctx.stroke();
        }
      });
    }

  }, [dbDimensions, samplePages, activePages]);

  function drawPageBox(ctx: CanvasRenderingContext2D, x: number, y: number, page: DbPage, isRoot: boolean, customWidth?: number) {
    const w = customWidth || (isRoot ? 90 : 65);
    const h = 28;

    // Glow effect for dirty pages
    if (page.dirty) {
      ctx.shadowBlur = 4;
      ctx.shadowColor = "rgba(0, 229, 255, 0.4)";
    }

    const isHighlighted = activePages.some(p => p.id === page.id);
    const borderColor = isHighlighted ? "#00e5ff" : page.dirty ? "#ffe600" : "#b44cff";
    const bgColor = isHighlighted ? "rgba(0, 229, 255, 0.08)" : "rgba(20, 8, 50, 0.9)";

    ctx.fillStyle = bgColor;
    ctx.strokeStyle = borderColor;
    ctx.lineWidth = isHighlighted ? 2 : 1;
    ctx.beginPath();
    ctx.roundRect?.(x, y, w, h, 3);
    ctx.fill();
    ctx.stroke();

    ctx.shadowBlur = 0;

    // Page ID and type label
    ctx.fillStyle = "#e4e4e7";
    ctx.font = "bold 7px var(--font-mono)";
    ctx.textAlign = "center";
    ctx.fillText(page.id, x + w / 2, y + 12);

    ctx.fillStyle = page.dirty ? "#ffe600" : "rgba(255,255,255,0.5)";
    ctx.font = "6px var(--font-mono)";
    ctx.fillText(page.type === 'ROOT_NODE' ? 'ROOT' : page.type === 'INTERNAL_NODE' ? 'INT' : page.type === 'LEAF_NODE' ? 'LEAF' : page.type, x + w / 2, y + 22);
  }

  return (
    <div className="flex flex-col gap-6">
      {/* Component A: Dual-Mode Console */}
      <div className="cyber-panel rounded overflow-hidden">
        {/* Mode Tabs */}
        <div className="flex border-b border-border">
          <button
            onClick={() => setMode('kv')}
            className={`flex-1 text-xs font-mono font-bold py-2.5 px-4 transition-all duration-150 tracking-wider ${
              mode === 'kv'
                ? 'bg-blue-bg text-blue border-b-2 border-blue'
                : 'text-text-soft hover:text-text hover:bg-border/30'
            }`}
          >
            $ KV_MUTATIONS // PUT GET DELETE
          </button>
          <button
            onClick={() => setMode('sql')}
            className={`flex-1 text-xs font-mono font-bold py-2.5 px-4 transition-all duration-150 tracking-wider ${
              mode === 'sql'
                ? 'bg-blue-bg text-blue border-b-2 border-blue'
                : 'text-text-soft hover:text-text hover:bg-border/30'
            }`}
          >
            ⟐ SQL_QUERY // B+Tree Parser
          </button>
        </div>

        {/* Console Content */}
        <div className="flex flex-col lg:flex-row">
          {/* Input Area */}
          <div className="flex-1 p-4 border-b lg:border-b-0 lg:border-r border-border">
            {mode === 'kv' ? (
              <div className="flex flex-col gap-3">
                <div className="text-[10px] font-mono text-text-soft font-bold tracking-wider mb-1">
                  [KV_STORE] // ENTER OPERATION (PUT &lt;key&gt; &lt;value&gt; | GET &lt;key&gt; | DELETE &lt;key&gt;)
                </div>
                <div className="flex gap-2">
                  <input
                    type="text"
                    value={kvInput}
                    onChange={(e) => setKvInput(e.target.value)}
                    onKeyDown={(e) => { if (e.key === 'Enter' && !e.shiftKey) { e.preventDefault(); handleExecute(); } }}
                    placeholder="PUT user:1001 JohnDoe"
                    className="flex-1 bg-bg border border-border rounded px-3 py-2 text-sm font-mono text-text placeholder-neutral-600 outline-none focus:border-blue/50 transition-colors"
                  />
                  <button
                    onClick={handleExecute}
                    disabled={queryRunning}
                    className="bg-cyan-900/30 border border-blue-border text-blue font-mono font-bold text-[10px] px-4 py-2 rounded hover:bg-cyan-900/50 transition-colors disabled:opacity-40"
                  >
                    EXEC
                  </button>
                </div>
              </div>
            ) : (
              <div className="flex flex-col gap-3">
                <div className="text-[10px] font-mono text-text-soft font-bold tracking-wider mb-1">
                  [SQL_ENGINE] // ENTER SQL STATEMENT (SELECT, INSERT, CREATE)
                </div>
                <textarea
                  value={sqlInput}
                  onChange={(e) => setSqlInput(e.target.value)}
                  onKeyDown={(e) => { if ((e.ctrlKey || e.metaKey) && e.key === 'Enter') { e.preventDefault(); handleExecute(); } }}
                  placeholder={`SELECT * FROM cluster_nodes\nWHERE status = 'Healthy'\nORDER BY cpu DESC;`}
                  rows={4}
                  className="flex-1 bg-bg border border-border rounded px-3 py-2 text-sm font-mono text-text placeholder-neutral-600 outline-none focus:border-blue/50 transition-colors resize-none"
                />
                <div className="flex justify-between items-center">
                  <span className="text-[9px] font-mono text-text-soft">Ctrl+Enter to execute</span>
                  <button
                    onClick={handleExecute}
                    disabled={queryRunning}
                    className="bg-cyan-900/30 border border-blue-border text-blue font-mono font-bold text-[10px] px-4 py-2 rounded hover:bg-cyan-900/50 transition-colors disabled:opacity-40"
                  >
                    {queryRunning ? 'RUNNING...' : '⟫ RUN QUERY'}
                  </button>
                </div>
              </div>
            )}
          </div>

          {/* Output Results */}
          <div className="w-full lg:w-[320px] xl:w-[400px] p-3 overflow-hidden">
            <div className="text-[9px] font-mono text-text-soft font-bold tracking-wider mb-2 border-b border-border pb-1">OUTPUT_BUFFER</div>
            <div
              ref={outputRef}
              className="h-[160px] overflow-y-auto font-mono text-[10px] leading-relaxed space-y-1"
            >
              {output.length === 0 ? (
                <span className="text-text-muted">// Awaiting query execution...</span>
              ) : (
                output.map((line, i) => (
                  <div key={i} className="whitespace-pre-wrap break-all">
                    {line
                      .replace(/\x1b\[36m/g, '<span class="text-blue">')
                      .replace(/\x1b\[32m/g, '<span class="text-green">')
                      .replace(/\x1b\[33m/g, '<span class="text-gold">')
                      .replace(/\x1b\[31m/g, '<span class="text-red">')
                      .replace(/\x1b\[35m/g, '<span class="text-purple-400">')
                      .replace(/\x1b\[0m/g, '</span>')
                      .replace(/\[/g, '<span class="text-text-soft">[')
                      .replace(/\]/g, ']</span>')
                    }
                  </div>
                ))
              )}
            </div>
          </div>
        </div>
      </div>

      {/* Component B: Database Page Structural Visualizer */}
      <div className="cyber-panel rounded overflow-hidden" ref={dbContainerRef}>
        <div className="flex items-center justify-between px-4 py-2 border-b border-border">
          <span className="text-xs font-mono font-bold text-text-soft tracking-wider">LMS_TREE // B+TREE PAGE STRUCTURAL MAP</span>
          <div className="flex gap-4">
            <span className="text-[9px] font-mono text-blue font-bold">● ACTIVE PAGE ACCESS</span>
            <span className="text-[9px] font-mono text-gold font-bold">◆ DIRTY (UNFLUSHED)</span>
          </div>
        </div>
        <div className="bg-bg w-full h-[240px] relative overflow-hidden">
          <canvas
            ref={dbCanvasRef}
            width={dbDimensions.width}
            height={dbDimensions.height}
            className="absolute inset-0 w-full h-full"
          />
        </div>
      </div>
    </div>
  );
}
