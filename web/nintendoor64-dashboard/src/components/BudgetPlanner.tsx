import React from "react";
import { Scale } from "lucide-react";

interface Metric {
  name: string;
  value: number;
  limit: number;
  unit: string;
  color: string;
}

const METRICS: Metric[] = [
  { name: "ROM Usage", value: 78, limit: 100, unit: "%", color: "#3b82f6" },
  { name: "RAM Usage", value: 42, limit: 64, unit: "MiB", color: "#10b981" },
  { name: "CPU Cycles", value: 68, limit: 100, unit: "%", color: "#f59e0b" },
  { name: "VRAM", value: 2.4, limit: 4, unit: "MiB", color: "#8b5cf6" }
];

export const BudgetPlanner: React.FC = () => {
  const [animated, setAnimated] = React.useState<number[]>(
    METRICS.map(() => 0)
  );

  React.useEffect(() => {
    const timer = setTimeout(
      () => setAnimated(METRICS.map((m) => (m.value / m.limit) * 100)),
      200
    );
    return () => clearTimeout(timer);
  }, []);

  return (
    <div className="card-bg border-glow rounded-xl p-6">
      <div className="flex items-center gap-3 mb-6">
        <div className="p-2 bg-blue-900/50 rounded-lg">
          <Scale className="text-blue-400" />
        </div>
        <div>
          <h3 className="text-lg font-semibold">Budget Planner</h3>
          <p className="text-sm text-slate-400">N64 constraints</p>
        </div>
      </div>
      {METRICS.map((m, idx) => (
        <div key={m.name} className="mb-4">
          <div className="flex justify-between text-sm mb-1">
            <span>{m.name}</span>
            <span className="font-mono">
              {m.value}
              {m.unit} / {m.limit}
              {m.unit}
            </span>
          </div>
          <div className="w-full bg-slate-800 rounded-full h-2.5">
            <div
              className="gauge-fill h-full rounded-full"
              style={{ width: `${animated[idx]}%`, backgroundColor: m.color }}
            />
          </div>
        </div>
      ))}
    </div>
  );
};
