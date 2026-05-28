type SkeletonProps = {
  rows?: number;
  variant?: "card" | "table" | "chart";
};

export default function Skeleton({ rows = 3, variant = "card" }: SkeletonProps) {
  if (variant === "chart") {
    return (
      <div className="cyber-panel rounded p-4 animate-pulse">
        <div className="h-4 w-32 bg-border rounded mb-4" />
        <div className="h-[180px] bg-border/40 rounded" />
      </div>
    );
  }

  if (variant === "table") {
    return (
      <div className="cyber-panel rounded overflow-hidden animate-pulse">
        <div className="px-3 py-2 border-b border-border">
          <div className="h-3 w-48 bg-border rounded" />
        </div>
        {Array.from({ length: rows }).map((_, i) => (
          <div key={i} className="px-3 py-3 border-b border-border/50">
            <div className="flex items-center gap-3 mb-1">
              <div className="h-2 w-2 rounded-full bg-border" />
              <div className="h-3 w-40 bg-border rounded" />
              <div className="h-3 w-14 bg-border rounded" />
            </div>
            <div className="flex gap-4">
              <div className="h-2 w-20 bg-border/40 rounded" />
              <div className="h-2 w-24 bg-border/40 rounded" />
            </div>
          </div>
        ))}
      </div>
    );
  }

  return (
    <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-3 animate-pulse">
      {Array.from({ length: rows }).map((_, i) => (
        <div key={i} className="cyber-panel rounded p-4">
          <div className="h-3 w-24 bg-border rounded mb-3" />
          <div className="h-5 w-16 bg-border/40 rounded mb-2" />
          <div className="h-2 w-32 bg-border/20 rounded" />
        </div>
      ))}
    </div>
  );
}
