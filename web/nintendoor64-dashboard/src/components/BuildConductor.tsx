import React from "react";
import { Workflow } from "lucide-react";

type Status = "completed" | "running" | "pending";

interface Step {
  id: number;
  name: string;
  status: Status;
  deps: number[];
}

const INITIAL_STEPS: Step[] = [
  { id: 1, name: "Codegen", status: "completed", deps: [] },
  { id: 2, name: "Asset Pack", status: "completed", deps: [1] },
  { id: 3, name: "Schema Check", status: "completed", deps: [1, 2] },
  { id: 4, name: "ROM Build", status: "running", deps: [3] },
  { id: 5, name: "Patch", status: "pending", deps: [4] },
  { id: 6, name: "Emulator", status: "pending", deps: [5] }
];

const NODE_POS: Record<number, [number, number]> = {
  1: [120, 160],
  2: [240, 80],
  3: [240, 240],
  4: [380, 160],
  5: [520, 160],
  6: [660, 160]
};

const STATUS_COLOR: Record<Status, string> = {
  completed: "#22c55e",
  running: "#3b82f6",
  pending: "#475569"
};

export const BuildConductor: React.FC = () => {
  const [steps, setSteps] = React.useState<Step[]>(INITIAL_STEPS);

  React.useEffect(() => {
    const timer = setTimeout(() => {
      setSteps((prev) =>
        prev.map((s) => (s.id === 4 ? { ...s, status: "completed" } : s))
      );
    }, 2000);
    return () => clearTimeout(timer);
  }, []);

  const edges: [number, number][] = [
    [1, 2],
    [1, 3],
    [2, 3],
    [3, 4],
    [4, 5],
    [5, 6]
  ];

  return (
    <div className="card-bg border-glow rounded-xl p-6">
      <div className="flex items-center gap-3 mb-4">
        <div className="p-2 bg-indigo-900/50 rounded-lg">
          <Workflow className="text-indigo-400" />
        </div>
        <div>
          <h3 className="text-lg font-semibold">Build Conductor</h3>
          <p className="text-sm text-slate-400">DAG pipeline</p>
        </div>
      </div>

      <svg viewBox="0 0 800 300" className="w-full">
        <defs>
          <marker
            id="arrow"
            markerWidth="8"
            markerHeight="6"
            refX="8"
            refY="3"
            orient="auto"
          >
            <polygon points="0 0, 8 3, 0 6" fill="#64748b" />
          </marker>
        </defs>

        {edges.map(([fromId, toId], i) => {
          const from = NODE_POS[fromId];
          const to = NODE_POS[toId];
          return (
            <line
              key={i}
              x1={from[0]}
              y1={from[1]}
              x2={to[0]}
              y2={to[1]}
              stroke="#64748b"
              strokeWidth={2}
              markerEnd="url(#arrow)"
              opacity={0.6}
            />
          );
        })}

        {steps.map((s) => {
          const [cx, cy] = NODE_POS[s.id];
          return (
            <g key={s.id}>
              <circle cx={cx} cy={cy} r={18} fill={STATUS_COLOR[s.status]} />
              <text
                x={cx}
                y={cy + 5}
                textAnchor="middle"
                fill="#fff"
                fontSize={10}
                fontWeight="bold"
              >
                {s.status === "completed"
                  ? "✓"
                  : s.status === "running"
                  ? "⟳"
                  : ""}
              </text>
              <text
                x={cx}
                y={cy + 35}
                textAnchor="middle"
                fill="#cbd5e1"
                fontSize={11}
              >
                {s.name}
              </text>
            </g>
          );
        })}
      </svg>
    </div>
  );
};
