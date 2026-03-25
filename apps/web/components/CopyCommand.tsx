"use client";

import { useState } from "react";

export default function CopyCommand({ command }: { command: string }) {
  const [copied, setCopied] = useState(false);

  const handleCopy = async () => {
    try {
      await navigator.clipboard.writeText(command);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    } catch (err) {
      console.error("Failed to copy!", err);
    }
  };

  return (
    <div className="group relative flex min-w-0 flex-col items-start gap-3 rounded-lg border border-zinc-200 bg-zinc-50/50 p-3 transition-all hover:border-primary/50 sm:flex-row sm:items-center sm:justify-between sm:p-4 dark:border-zinc-700 dark:bg-zinc-900/50">
      <code className="block w-full min-w-0 overflow-x-auto whitespace-nowrap font-mono text-sm text-slate-900 dark:text-slate-100  no-scrollbar">
        <span className="font-bold text-primary">$</span> {command}
      </code>
      <button
        onClick={handleCopy}
        className={[
          "shrink-0 text-xs font-bold uppercase tracking-widest transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-sky-500",
          copied
            ? "text-green-500"
            : "text-zinc-500 hover:text-primary dark:text-zinc-400",
        ].join(" ")}
      >
        {copied ? "Copied!" : "Copy"}
      </button>
    </div>
  );
}
