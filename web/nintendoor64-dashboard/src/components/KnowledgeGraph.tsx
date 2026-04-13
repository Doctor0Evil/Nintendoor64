import React from "react";
import { Network } from "lucide-react";

type NodeType = "tool" | "resource" | "component";

interface Node {
  id: string;
  name: string;
  type: NodeType;
  x: number;
  y: number;
  color: string;
}

const NODES: Node[] = [
  { id: "starzip", name: "starzip", type: "tool", x: 150, y: 120, color: "#3b82f6" },
  { id: "gamemodeai", name: "gamemodeai", type: "tool", x: 300, y: 60, color: "#3b82f6" },
  { id: "schemas", name: "schemas", type: "resource", x: 150, y: 220, color: "#22c55e" },
  { id: "session", name: "session", type: "component", x: 450, y: 150, color: "#8b5cf6" },
  { id: "kg", name: "KG", type: "resource", x: 300, y: 240, color: "#22c55e" }
];

const EDGES: [string, string][] = [
  ["schemas", "starzip"],
  ["kg", "starzip"],
  ["schemas", "gamemodeai"],
  ["starzip", "session"],
  ["gamemodeai", "session"]
];

export const KnowledgeGraph: React.FC = () => {
  return (
    <div className="card-bg border-glow rounded-xl p-6">
      <div className="flex items-center gap-3 mb-4">
        <div className="p-2 bg-indigo-900/50 rounded-lg">
          <Network className="text-indigo-400" />
        </div>
        <div>
          <h3 className="text-lg font-semibold">Knowledge Graph</h3>
          <p className="text-sm text-slate-400">System topology</p>
        </div>
      </div>

      <svg viewBox="0 0 600 300" className="w-full">
        {EDGES.map(([fromId, toId], i) => {
          const from = NODES.find((n) => n.id === fromId)!;
          const to = NODES.find((n) => n.id === toId)!;
          return (
            <line
              key={`${fromId}-${toId}-${i}`}
              x1={from.x}
              y1={from.y}
              x2={to.x}
              y2={to.y}
              stroke="#64748b"
              strokeWidth={2}
              opacity={0.5}
            />
          );
        })}
        {NODES.map((n) => (
          <g key={n.id}>
            <circle cx={n.x} cy={n.y} r={20} fill={n.color} />
            <text
              x={n.x}
              y={n.y + 5}
              textAnchor="middle"
              fill="#fff"
              fontSize={10}
            >
              {n.name}
            </text>
          </g>
        ))}
      </svg>
    </div>
  );
};
