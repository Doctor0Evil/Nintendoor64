import React from "react";
import { ShieldCheck, AlertOctagon } from "lucide-react";

interface InvariantRow {
  name: string;
  pass: boolean;
}

const INVARIANTS: InvariantRow[] = [
  { name: "ROM < 32 MiB", pass: true },
  { name: "No patch overlaps", pass: true },
  { name: "Boot hook valid", pass: false },
  { name: "Texture budget OK", pass: true }
];

export const SessionConditioning: React.FC = () => {
  const score = 87;

  return (
    <div className="card-bg border-glow rounded-xl p-6">
      <div className="flex items-center gap-3 mb-6">
        <div className="p-2 bg-amber-900/50 rounded-lg">
          <ShieldCheck className="text-amber-400" />
        </div>
        <div>
          <h3 className="text-lg font-semibold">Session Conditioning</h3>
          <p className="text-sm text-slate-400">Score: {score}</p>
        </div>
      </div>

      <div className="space-y-2">
        {INVARIANTS.map((inv) => (
          <div
            key={inv.name}
            className="flex items-center justify-between p-2 bg-slate-800/50 rounded"
          >
            <span className="text-sm">{inv.name}</span>
            <span
              className={`text-xs px-2 py-0.5 rounded ${
                inv.pass
                  ? "bg-green-900/50 text-green-400"
                  : "bg-red-900/50 text-red-400"
              }`}
            >
              {inv.pass ? "PASS" : "FAIL"}
            </span>
          </div>
        ))}
      </div>

      <div className="mt-4 p-3 bg-red-900/20 border border-red-800 rounded text-xs text-red-300 flex items-center gap-2">
        <AlertOctagon size={14} />
        <span>Boot hook overlaps with Code segment</span>
      </div>
    </div>
  );
};
