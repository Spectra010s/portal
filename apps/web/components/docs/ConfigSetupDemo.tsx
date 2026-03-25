"use client";

import { useState } from "react";

const steps = [
  {
    id: "username",
    label: "Username",
    prompt: "? What is your username?",
    help: "This identifies you during transfers. Press Enter to keep the random suggestion.",
    answer: "Username: spectra@portal",
    note: "Portal appends `@portal` if you do not type it yourself.",
  },
  {
    id: "port",
    label: "Port",
    prompt: "? Which port should Portal use?",
    help: "The local port used for listening. 7878 is recommended.",
    answer: "7878",
    note: "Ports must be greater than 1024 to avoid system conflicts.",
  },
  {
    id: "dir",
    label: "Download Dir",
    prompt: "? Where should Portal save downloaded files?",
    help: "Enter a valid folder path.",
    answer: "/home/spectra/Downloads",
    note: "The code defaults to your home Downloads folder.",
  },
  {
    id: "done",
    label: "Saved Config",
    prompt: "Configuration saved. You're ready to use Portal.",
    help: "Portal writes the file to ~/.portal/config.toml.",
    answer: `[user]\nusername = "spectra@portal"\n\n[network]\ndefault_port = 7878\n\n[storage]\ndownload_dir = "/home/spectra/Downloads"`,
    note: "If config.toml already exists, Portal asks whether you want to overwrite it before running setup.",
  },
];

export default function ConfigSetupDemo() {
  const [activeStep, setActiveStep] = useState(steps[0]);

  return (
    <div className="my-8 overflow-hidden rounded-[1.75rem] border border-slate-200 bg-slate-950 text-slate-100 shadow-[0_30px_80px_-40px_rgba(2,6,23,0.9)]">
      <div className="flex flex-wrap items-center justify-between gap-2 border-b border-slate-800 bg-slate-900 px-4 py-3">
        <div className="flex items-center gap-2">
          <span className="h-3 w-3 rounded-full bg-red-500" />
          <span className="h-3 w-3 rounded-full bg-yellow-400" />
          <span className="h-3 w-3 rounded-full bg-green-500" />
        </div>
        <p className="text-[11px] font-semibold uppercase tracking-[0.24em] text-slate-400">
          portal config setup
        </p>
      </div>

      <div className="border-b border-slate-800 px-4 py-3">
        <div className="flex flex-wrap gap-2">
          {steps.map((step) => {
            const active = step.id === activeStep.id;

            return (
              <button
                key={step.id}
                onClick={() => setActiveStep(step)}
                className={[
                  "rounded-full px-3 py-1.5 text-xs font-semibold transition",
                  active
                    ? "bg-sky-500 text-white"
                    : "bg-slate-800 text-slate-300 hover:bg-slate-700",
                ].join(" ")}
              >
                {step.label}
              </button>
            );
          })}
        </div>
      </div>

      <div className="grid gap-0 lg:grid-cols-[minmax(0,1fr)_280px]">
        <div className="min-w-0 border-b border-slate-800 p-4 lg:border-b-0 lg:border-r lg:p-5">
          <div className="overflow-x-auto rounded-2xl bg-black/30 p-3 font-mono text-sm leading-7 text-slate-200 sm:p-4">
            <div className="min-w-max">
              <p className="text-sky-400">$ portal config setup</p>
              <p className="mt-3 text-slate-100">
                Welcome to Portal! Let&apos;s get you set up.
              </p>
              <p className="mt-4 text-slate-300">{activeStep.prompt}</p>
              <p className="text-xs text-slate-500">{activeStep.help}</p>
              <pre className="mt-4 overflow-x-auto rounded-xl border border-slate-800 bg-slate-950 p-4 text-sm text-emerald-300">
                {activeStep.answer}
              </pre>
            </div>
          </div>
        </div>

        <div className="p-4 lg:p-5">
          <p className="text-[11px] font-semibold uppercase tracking-[0.24em] text-slate-500">
            What this step means
          </p>
          <p className="mt-3 text-sm leading-7 text-slate-300">
            {activeStep.note}
          </p>
        </div>
      </div>
    </div>
  );
}
