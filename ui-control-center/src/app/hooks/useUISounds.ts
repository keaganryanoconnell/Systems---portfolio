"use client";

import { useCallback, useRef } from "react";

type SoundName = "pop" | "click" | "whoosh" | "complete";

let audioCtx: AudioContext | null = null;
let _muted = true;

function getCtx(): AudioContext {
  if (!audioCtx) {
    audioCtx = new (window.AudioContext || (window as any).webkitAudioContext)();
  }
  return audioCtx;
}

export function isMuted(): boolean {
  return _muted;
}

export function toggleMute(): boolean {
  _muted = !_muted;
  return _muted;
}

export function useUISounds() {
  const play = useCallback((name: SoundName) => {
    if (_muted) return;
    try {
      const ctx = getCtx();
      const now = ctx.currentTime;
      const gain = ctx.createGain();
      gain.connect(ctx.destination);
      gain.gain.setValueAtTime(0.04, now);

      switch (name) {
        case "pop": {
          const osc = ctx.createOscillator();
          osc.type = "sine";
          osc.frequency.setValueAtTime(200, now);
          osc.frequency.exponentialRampToValueAtTime(400, now + 0.04);
          osc.connect(gain);
          gain.gain.exponentialRampToValueAtTime(0.001, now + 0.06);
          osc.start(now);
          osc.stop(now + 0.06);
          break;
        }
        case "click": {
          const osc = ctx.createOscillator();
          osc.type = "square";
          osc.frequency.setValueAtTime(800, now);
          osc.connect(gain);
          gain.gain.setValueAtTime(0.03, now);
          gain.gain.exponentialRampToValueAtTime(0.001, now + 0.03);
          osc.start(now);
          osc.stop(now + 0.03);
          break;
        }
        case "whoosh": {
          const bufferSize = ctx.sampleRate * 0.2;
          const buffer = ctx.createBuffer(1, bufferSize, ctx.sampleRate);
          const data = buffer.getChannelData(0);
          for (let i = 0; i < bufferSize; i++) {
            data[i] = (Math.random() * 2 - 1) * (1 - i / bufferSize);
          }
          const src = ctx.createBufferSource();
          src.buffer = buffer;
          const filter = ctx.createBiquadFilter();
          filter.type = "bandpass";
          filter.frequency.setValueAtTime(2000, now);
          filter.Q.setValueAtTime(0.5, now);
          src.connect(filter);
          filter.connect(gain);
          gain.gain.setValueAtTime(0.04, now);
          gain.gain.exponentialRampToValueAtTime(0.001, now + 0.2);
          src.start(now);
          break;
        }
        case "complete": {
          const notes = [523, 659, 784];
          notes.forEach((freq, i) => {
            const osc = ctx.createOscillator();
            osc.type = "sine";
            osc.frequency.setValueAtTime(freq, now + i * 0.1);
            const g = ctx.createGain();
            g.connect(ctx.destination);
            g.gain.setValueAtTime(0.05, now + i * 0.1);
            g.gain.exponentialRampToValueAtTime(0.001, now + i * 0.1 + 0.3);
            osc.connect(g);
            osc.start(now + i * 0.1);
            osc.stop(now + i * 0.1 + 0.3);
          });
          break;
        }
      }
    } catch {
      // Audio not available
    }
  }, []);

  return { play };
}
