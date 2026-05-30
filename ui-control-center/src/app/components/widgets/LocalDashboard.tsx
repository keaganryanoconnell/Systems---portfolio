"use client";

import { useEffect, useState } from "react";
import { Cloud, Clock } from "lucide-react";

export default function LocalDashboard() {
  const [time, setTime] = useState("");
  const [weather, setWeather] = useState<{ temp: string; condition: string; humidity: string } | null>(null);
  const [timezone, setTimezone] = useState("");
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    const tz = Intl.DateTimeFormat().resolvedOptions().timeZone;
    setTimezone(tz);

    const tick = () => {
      setTime(
        new Date().toLocaleTimeString("en-US", {
          hour: "2-digit",
          minute: "2-digit",
          second: "2-digit",
          hour12: false,
        })
      );
    };
    tick();
    const interval = setInterval(tick, 1000);

    fetch("https://wttr.in/?format=j1")
      .then((r) => r.json())
      .then((data: any) => {
        const c = data.current_condition?.[0];
        if (c) {
          setWeather({
            temp: `${c.temp_C}°C`,
            condition: c.weatherDesc?.[0]?.value || "Clear",
            humidity: `${c.humidity}%`,
          });
        }
        setLoading(false);
      })
      .catch(() => setLoading(false));

    return () => clearInterval(interval);
  }, []);

  return (
    <div className="cyber-panel p-4 border-l-4 border-l-blue">
      <div className="flex items-center gap-2 mb-3 pb-2 border-b border-border">
        <Clock size={14} className="text-blue" />
        <span className="text-[9px] font-mono font-bold text-blue tracking-wider uppercase">
          Local Dashboard
        </span>
      </div>
      <div className="space-y-3">
        <div>
          <div className="text-2xl font-mono font-bold text-text tracking-tight">{time}</div>
          <div className="text-[9px] font-mono text-text-muted mt-0.5">{timezone}</div>
        </div>
        {weather && (
          <div className="flex items-center gap-3">
            <Cloud size={20} className="text-text-soft" />
            <div>
              <div className="text-[10px] font-mono font-bold text-text">{weather.temp}</div>
              <div className="text-[9px] font-mono text-text-muted">{weather.condition} · {weather.humidity}</div>
            </div>
          </div>
        )}
        {loading && (
          <div className="text-[10px] font-mono text-text-muted animate-pulse">Loading weather...</div>
        )}
      </div>
    </div>
  );
}
