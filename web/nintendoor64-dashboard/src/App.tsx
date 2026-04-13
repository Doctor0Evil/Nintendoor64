import React from "react";
import {
  Cpu,
  Layout,
  FileJson,
  Workflow,
  Layers,
  Gamepad2,
  ShieldCheck,
  Network
} from "lucide-react";
import { RomLayoutOracle } from "./components/RomLayoutOracle";
import { SafePatchSynthesizer } from "./components/SafePatchSynthesizer";
import { BudgetPlanner } from "./components/BudgetPlanner";
import { BuildConductor } from "./components/BuildConductor";
import { ScenarioDirector } from "./components/ScenarioDirector";
import { SessionConditioning } from "./components/SessionConditioning";
import { KnowledgeGraph } from "./components/KnowledgeGraph";

type TabId = "overview" | "design" | "build" | "analysis";

const tabs: { id: TabId; name: string; icon: React.ReactNode }[] = [
  { id: "overview", name: "Overview", icon: <Layout size={16} /> },
  { id: "design", name: "Design", icon: <FileJson size={16} /> },
  { id: "build", name: "Build", icon: <Workflow size={16} /> },
  { id: "analysis", name: "Analysis", icon: <Layers size={16} /> }
];

export const App: React.FC = () => {
  const [tab, setTab] = React.useState<TabId>("overview");

  return (
    <div className="min-h-screen bg-slate-950">
      <header className="border-b border-slate-800 bg-slate-900/70 sticky top-0 backdrop-blur">
        <div className="max-w-7xl mx-auto px-6 py-4 flex justify-between items-center">
          <div className="flex items-center gap-3">
            <div className="p-2 bg-gradient-to-br from-indigo-600 to-purple-700 rounded">
              <Cpu className="text-white" size={24} />
            </div>
            <div>
              <h1 className="text-2xl font-bold gradient-text">Nintendoor64</h1>
              <p className="text-xs text-slate-400">
                Contract-Driven Game Studio
              </p>
            </div>
          </div>
          <div className="px-3 py-1.5 bg-indigo-900/30 rounded text-xs font-mono text-indigo-300">
            sonia-server:active
          </div>
        </div>
        <div className="max-w-7xl mx-auto px-6 flex gap-1">
          {tabs.map((t) => (
            <button
              key={t.id}
              onClick={() => setTab(t.id)}
              className={`px-4 py-2 text-sm font-medium flex items-center gap-1 border-b-2 ${
                tab === t.id
                  ? "border-indigo-500 text-indigo-400"
                  : "border-transparent text-slate-400"
              }`}
            >
              {t.icon}
              {t.name}
            </button>
          ))}
        </div>
      </header>

      <main className="max-w-7xl mx-auto px-6 py-8">
        {tab === "overview" && (
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
            <RomLayoutOracle />
            <SafePatchSynthesizer />
            <BudgetPlanner />
            <BuildConductor />
            <ScenarioDirector />
            <SessionConditioning />
            <KnowledgeGraph />
          </div>
        )}

        {tab === "design" && (
          <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
            <KnowledgeGraph />
            <div className="card-bg border-glow p-6 rounded-xl">
              <h3 className="text-lg font-semibold mb-2">Schema Designer (mock)</h3>
              <p className="text-sm text-slate-400">
                This panel will render JSON-Schema driven forms for RomLayout,
                PatchSpec, and MissionDAG once wired to sonia-server.
              </p>
            </div>
          </div>
        )}

        {tab === "build" && (
          <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
            <BuildConductor />
            <SessionConditioning />
          </div>
        )}

        {tab === "analysis" && (
          <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
            <RomLayoutOracle />
            <SafePatchSynthesizer />
            <BudgetPlanner />
            <ScenarioDirector />
          </div>
        )}
      </main>
    </div>
  );
};
