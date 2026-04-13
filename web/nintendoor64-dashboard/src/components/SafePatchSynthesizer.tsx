import React from "react";
import { GitBranch } from "lucide-react";

type ChangeType = "add" | "del" | "unchanged";

interface Change {
  type: ChangeType;
  line: number;
  code: string;
}

const CHANGES: Change[] = [
  { type: "add", line: 1, code: "+ BOOT_HOOK(0x001000) → InitModernUI()" },
  { type: "del", line: 2, code: "- BOOT_HOOK(0x001000) → SkipIntroSequence()" },
  { type: "add", line: 3, code: "+ TEXTURE(\"player.ci8\", VRAM_0x800) → LZSS" },
  { type: "unchanged", line: 4, code: "  DATA(\"level_1.dat\") → Unchanged" },
  { type: "add", line: 5, code: "+ AUDIO(\"theme.mid\", 0xA000) → ADPCMA" },
  { type: "del", line: 6, code: "- AUDIO(\"theme.mid\", 0xA000) → RawPCM" }
];

export const SafePatchSynthesizer: React.FC = () => {
  return (
    <div className="card-bg border-glow rounded-xl p-6">
      <div className="flex items-center gap-3 mb-4">
        <div className="p-2 bg-green-900/50 rounded-lg">
          <GitBranch className="text-green-400" />
        </div>
        <div>
          <h3 className="text-lg font-semibold">Safe Patch Synthesizer</h3>
          <p className="text-sm text-slate-400">Byte-level diff preview</p>
        </div>
      </div>
      <div className="bg-slate-900/50 rounded-lg p-3 font-mono text-sm space-y-1">
        {CHANGES.map((c) => (
          <div
            key={c.line}
            className={`px-3 py-1.5 rounded text-xs ${
              c.type === "add"
                ? "diff-add text-green-300"
                : c.type === "del"
                ? "diff-del text-red-300"
                : "text-slate-400"
            }`}
          >
            <span className="opacity-50 mr-3">{c.line}</span>
            {c.code}
          </div>
        ))}
      </div>
    </div>
  );
};
