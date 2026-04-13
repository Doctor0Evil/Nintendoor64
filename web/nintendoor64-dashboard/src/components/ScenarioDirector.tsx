import React from "react";
import { Gamepad2 } from "lucide-react";

interface TelemetryPoint {
  awareness: number;
  fps: number;
}

const FRAMES = 100;

export const ScenarioDirector: React.FC = () => {
  const [frame, setFrame] = React.useState(0);
  const [playing, setPlaying] = React.useState(false);

  const data = React.useMemo<TelemetryPoint[]>(
    () =>
      Array.from({ length: FRAMES }, (_, i) => ({
        awareness: 20 + 40 * Math.sin(i / 10) + 5 * Math.random(),
        fps: 58 + 2 * Math.sin(i / 5)
      })),
    []
  );

  React.useEffect(() => {
    if (!playing) return;
    const iv = setInterval(
      () => setFrame((f) => (f >= FRAMES - 1 ? 0 : f + 1)),
      60
    );
    return () => clearInterval(iv);
  }, [playing]);

  const current = data[frame] ?? { awareness: 0, fps: 0 };

  return (
    <div className="card-bg border-glow rounded-xl p-6">
      <div className="flex items-center justify-between mb-4">
        <div className="flex items-center gap-3">
          <div className="p-2 bg-purple-900/50 rounded-lg">
            <Gamepad2 className="text-purple-400" />
          </div>
          <div>
            <h3 className="text-lg font-semibold">Scenario Director</h3>
            <p className="text-sm text-slate-400">Awareness telemetry</p>
          </div>
        </div>
        <button
          onClick={() => setPlaying(!playing)}
          className="px-3 py-1.5 bg-indigo-600 rounded text-sm"
        >
          {playing ? "Pause" : "Play"}
        </button>
      </div>

      <svg viewBox="0 0 400 120" className="w-full bg-slate-900/40 rounded">
        <polyline
          fill="none"
          stroke="#a78bfa"
          strokeWidth={2}
          points={data
            .slice(0, frame + 1)
            .map((d, i) => `${i * 4},${120 - d.awareness}`)
            .join(" ")}
        />
        <circle
          cx={frame * 4}
          cy={120 - current.awareness}
          r={4}
          fill="#c084fc"
        />
        <line
          x1={frame * 4}
          y1={0}
          x2={frame * 4}
          y2={120}
          stroke="#c084fc"
          strokeDasharray="3"
          opacity={0.4}
        />
      </svg>

      <div className="grid grid-cols-2 gap-3 mt-4">
        <div className="bg-slate-800/50 p-2 rounded">
          <span className="text-xs text-slate-400">Awareness</span>
          <div className="text-lg text-purple-400">
            {current.awareness.toFixed(1)}
          </div>
        </div>
        <div className="bg-slate-800/50 p-2 rounded">
          <span className="text-xs text-slate-400">FPS</span>
          <div className="text-lg text-green-400">
            {current.fps.toFixed(1)}
          </div>
        </div>
      </div>
    </div>
  );
};
