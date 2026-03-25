"use client";

import { useState } from "react";

const MODES = {
  discovery: {
    label: "Discovery Mode",
    senderTitle: "$ portal send --to spectra@portal ./assets",
    senderLines: [
      "Portal: Searching for receiver...: spectra@portal",
      "Portal: Connecting to 192.168.1.20:7878...",
      "Portal: Connection established!",
      "Portal: Verifying identity...",
      "Portal: Identity verified. Starting transfer...",
      "? Portal: Add description for this transfer? Yes",
      '? Portal: Enter transfer description: "Design handoff assets"',
      "Portal: Transfer initialized (2 files, 1 folders)",
      "Portal: Note: Design handoff assets",
      "Portal: Preparing to send 3 items(s)...",
      "Portal: All file(s) have been sent successfully!",
    ],
    receiverTitle: "$ portal receive",
    receiverLines: [
      "Portal: Creating wormhole at 192.168.1.20",
      "Portal: Wormhole open for spectra@portal",
      "Portal: Connection established with 192.168.1.44:53120!",
      "Portal: Connected to sender",
      "Portal: Waiting for incoming files...",
      "Portal: Incoming transfer - 3 item(s)",
      'Portal: Sender left a note: "Design handoff assets"',
      "Portal: Using directory from config: /home/spectra/Downloads",
      "Portal: All item(s) have been received successfully! Saved to '/home/spectra/Downloads'",
    ],
    note: "Discovery listens for multicast beacons, finds the receiver by username, then verifies the node ID the receiver claims over TCP before the transfer starts.",
  },
  direct: {
    label: "Direct IP Mode",
    senderTitle: "$ portal send --address 192.168.1.20 --port 7878 ./assets",
    senderLines: [
      "Portal: Connecting to 192.168.1.20:7878...",
      "Portal: Connection established!",
      "Portal: Connected to 192.168.1.20 (Manual mode: Identity check skipped).",
      "? Portal: Add description for this transfer? No",
      "Portal: Transfer initialized (2 files, 1 folders)",
      "Portal: Preparing to send 3 items(s)...",
      "Portal: All file(s) have been sent successfully!",
    ],
    receiverTitle: "$ portal receive --port 7878",
    receiverLines: [
      "Portal: Creating wormhole at 192.168.1.20",
      "Portal: Wormhole open for spectra@portal",
      "Portal: Connection established with 192.168.1.44:53120!",
      "Portal: Connected to sender",
      "Portal: Waiting for incoming files...",
      "Portal: Incoming transfer - 3 item(s)",
      "Portal: All item(s) have been received successfully! Saved to '/home/spectra/Downloads'",
    ],
    note: "Direct IP skips the node-ID verification path entirely. It is useful when you already trust the destination and just want a manual connection path.",
  },
} as const;

export default function TransferFlowDemo() {
  const [mode, setMode] = useState<keyof typeof MODES>("discovery");
  const current = MODES[mode];

  return (
    <div className="my-8 overflow-hidden rounded-[1.75rem] border border-slate-200 bg-slate-950 text-slate-100 shadow-[0_30px_80px_-40px_rgba(2,6,23,0.9)]">
      <div className="flex flex-wrap items-center justify-between gap-3 border-b border-slate-800 bg-slate-900 px-4 py-3">
        <p className="text-[11px] font-semibold uppercase tracking-[0.24em] text-slate-400">
          Portal Transfer Walkthrough
        </p>
        <div className="flex flex-wrap gap-2">
          {Object.entries(MODES).map(([key, value]) => (
            <button
              key={key}
              onClick={() => setMode(key as keyof typeof MODES)}
              className={[
                "rounded-full px-3 py-1.5 text-xs font-semibold transition",
                mode === key
                  ? "bg-sky-500 text-white"
                  : "bg-slate-800 text-slate-300 hover:bg-slate-700",
              ].join(" ")}
            >
              {value.label}
            </button>
          ))}
        </div>
      </div>

      <div className="grid gap-px bg-slate-800 lg:grid-cols-2">
        <section className="min-w-0 bg-slate-950 p-4 sm:p-5">
          <p className="text-[11px] font-semibold uppercase tracking-[0.24em] text-slate-500">
            Sender
          </p>
          <div className="mt-3 overflow-x-auto rounded-2xl bg-black/30 p-3 font-mono text-sm leading-7 text-slate-200 sm:p-4">
            <div className="min-w-max">
              <p className="text-sky-400">{current.senderTitle}</p>
              {current.senderLines.map((line) => (
                <p key={line} className="text-slate-300">
                  {line}
                </p>
              ))}
            </div>
          </div>
        </section>

        <section className="min-w-0 bg-slate-950 p-4 sm:p-5">
          <p className="text-[11px] font-semibold uppercase tracking-[0.24em] text-slate-500">
            Receiver
          </p>
          <div className="mt-3 overflow-x-auto rounded-2xl bg-black/30 p-3 font-mono text-sm leading-7 text-slate-200 sm:p-4">
            <div className="min-w-max">
              <p className="text-emerald-400">{current.receiverTitle}</p>
              {current.receiverLines.map((line) => (
                <p key={line} className="text-slate-300">
                  {line}
                </p>
              ))}
            </div>
          </div>
        </section>
      </div>

      <div className="border-t border-slate-800 px-4 py-4 text-sm leading-7 text-slate-300 sm:px-5">
        {current.note}
      </div>
    </div>
  );
}
