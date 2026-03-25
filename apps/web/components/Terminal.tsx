"use client";

import { useState } from "react";
import CopyCommand from "./CopyCommand";

const installers = [
  {
    id: "linux",
    label: "Linux / macOS",
    cmd: "curl -fsSL https://portal.build.app/install.sh | sh",
  },
  {
    id: "android",
    label: "Android / Termux",
    cmd: "curl -fsSL https://portal.build.app/install.sh | sh",
  },
  {
    id: "windows",
    label: "PowerShell",
    cmd: 'powershell -ExecutionPolicy Bypass -c "irm https://portal.build.app/install.ps1 | iex"',
  },
  { id: "npm", label: "npm", cmd: "npm install -g @hiverra/portal@0.10.1" },
];

export default function InstallTerminal() {
  const [activeTab, setActiveTab] = useState("linux");

  return (
    <div className="w-full overflow-hidden rounded-[1.5rem] border border-slate-200 bg-white shadow-2xl dark:border-slate-800 dark:bg-slate-900">
      {/* Terminal Header */}
      <div className="flex items-center gap-2 border-b border-slate-100 bg-slate-50 px-4 py-3 dark:border-slate-800 dark:bg-slate-950">
        <div className="flex gap-1.5">
          <div className="h-3 w-3 rounded-full border border-red-600 bg-red-500" />
          <div className="h-3 w-3 rounded-full border border-yellow-500 bg-yellow-400" />
          <div className="h-3 w-3 rounded-full border border-green-600 bg-green-500" />
        </div>
        <div className="ml-4 text-[10px] font-bold uppercase tracking-widest text-slate-500 dark:text-slate-400">
          Install Portal v0.10.1
        </div>
      </div>

      <div className="flex overflow-x-auto no-scrollbar border-b border-slate-100 bg-white whitespace-nowrap dark:border-slate-800 dark:bg-slate-900">
        {installers.map((tab) => {
          const active = activeTab === tab.id;
          return (
            <button
              key={tab.id}
              onClick={() => setActiveTab(tab.id)}
              className={[
                "shrink-0 px-5 py-3 text-[11px] font-bold uppercase tracking-wider transition-all focus-visible:outline-none",
                active
                  ? "border-b-2 border-primary text-primary"
                  : "text-slate-500 hover:text-slate-800 dark:text-slate-400 dark:hover:text-slate-200",
              ].join(" ")}
            >
              {tab.label}
            </button>
          );
        })}
      </div>

      <div className="p-4 sm:p-6">
        {installers.map(
          (tab) =>
            tab.id === activeTab && (
              <div
                key={tab.id}
                className=" animate-in fade-in slide-in-from-bottom-1 duration-300"
              >
                <CopyCommand command={tab.cmd} />
                <p className="mt-4 text-[11px] font-medium text-slate-500 dark:text-slate-400">
                  Run this in your terminal to install Portal.
                </p>
              </div>
            ),
        )}
      </div>
    </div>
  );
}
