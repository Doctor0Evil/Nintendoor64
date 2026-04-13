import React from "react";
import { Layers } from "lucide-react";

type Segment = {
  name: string;
  offset: string;
  size: number;
  color: string;
  usage: string;
};

const SEGMENTS: Segment[] = [
  { name: "Boot", offset: "0x000000", size: 0x10000, color: "#6366f1", usage: "Boot code" },
  { name: "Code", offset: "0x001000", size: 0x80000, color: "#8b5cf6", usage: "Main game code" },
  { name: "Textures", offset: "0x081000", size: 0x120000, color: "#3b82f6", usage: "Asset textures" },
  { name: "Audio", offset: "0x1A1000", size: 0x180000, color: "#10b981", usage: "Music & SFX" },
  { name: "Data", offset: "0x321000", size: 0x0D0000, color: "#f59e0b", usage: "Level data" },
  { name: "Patches", offset: "0x3F1000", size: 0x00F000, color: "#ef4444", usage: "AI modifications" }
];

const totalSize = SEGMENTS.reduce((sum, s) => sum + s.size, 0);

export const RomLayoutOracle: React.FC = () => {
  const [hoveredIdx, setHoveredIdx] = React.useState<number | null>(null);

  return (
    <div className="card-bg border-glow rounded-xl p-6">
      <div className="flex items-center justify-between mb-6">
        <div className="flex items-center gap-3">
          <div className="p-2 bg-indigo-900/50 rounded-lg">
            <Layers className="text-indigo-400" />
          </div>
          <div>
            <h3 className="text-lg font-semibold">ROM Layout Oracle</h3>
            <p className="text-sm text-slate-400">Interactive segment map</p>
          </div>
        </div>
        <div className="px-3 py-1 bg-indigo-900/30 rounded-full text-xs font-mono">
          4.0 MiB
        </div>
      </div>

      <div className="space-y-3">
        {SEGMENTS.map((seg, idx) => {
          const widthPct = (seg.size / totalSize) * 100;
          return (
            <div
              key={seg.name}
              className="segment-row rounded-lg p-2 -mx-2"
              onMouseEnter={() => setHoveredIdx(idx)}
              onMouseLeave={() => setHoveredIdx(null)}
            >
              <div className="flex justify-between items-center mb-1">
                <div className="flex items-center gap-2">
                  <div
                    className="w-3 h-3 rounded-full"
                    style={{ backgroundColor: seg.color }}
                  />
                  <span className="text-sm font-medium">{seg.name}</span>
                </div>
                <span className="text-xs font-mono text-slate-400">
                  {seg.offset}
                </span>
              </div>

              <div className="w-full bg-slate-800 rounded-full h-6 overflow-hidden">
                <div
                  className="h-full rounded-full relative"
                  style={{ width: `${widthPct}%`, backgroundColor: seg.color }}
                >
                  {hoveredIdx === idx && (
                    <div className="absolute right-2 top-1/2 -translate-y-1/2 text-xs font-bold text-white">
                      {widthPct.toFixed(1)}%
                    </div>
                  )}
                </div>
              </div>

              {hoveredIdx === idx && (
                <div className="mt-2 p-2 bg-slate-800/70 rounded text-xs">
                  <div className="flex justify-between">
                    <span>Size:</span>
                    <span className="font-mono">
                      {(seg.size / 1024).toFixed(0)} KiB
                    </span>
                  </div>
                  <div className="flex justify-between">
                    <span>Usage:</span>
                    <span>{seg.usage}</span>
                  </div>
                </div>
              )}
            </div>
          );
        })}
      </div>
    </div>
  );
};
