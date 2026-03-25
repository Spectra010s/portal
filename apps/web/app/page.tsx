"use client";

import Image from "next/image";
import Link from "next/link";
import InstallTerminal from "@/components/Terminal";
import { Radar, Network, Folder } from "lucide-react";
import Footer from "@/components/Footer";
import { motion } from "framer-motion";

export default function Home() {
  const highlights = [
    {
      title: "Verified local discovery",
      desc: "Find receivers on the same network and confirm who you are sending to before the transfer starts.",
      icon: Radar,
    },
    {
      title: "Direct IP when speed matters",
      desc: "Jump straight to an address and port when you already know the target environment.",
      icon: Network,
    },
    {
      title: "Folders, history, and exports",
      desc: "Send recursively, inspect transfer records, and export history when you need an audit trail.",
      icon: Folder,
    },
  ];

  // Animation Variants
  const fadeInUp = {
    initial: { opacity: 0, y: 20 },
    animate: { opacity: 1, y: 0 },
    transition: { duration: 0.5, ease: [0.19, 1, 0.22, 1] },
  };

  const staggerContainer = {
    animate: {
      transition: {
        staggerChildren: 0.1,
      },
    },
  };

  return (
    <div className="flex min-h-screen flex-col font-sans text-foreground px-4 py-8 md:py-16 bg-background">
      <main className="mx-auto flex w-full max-w-8xl flex-col lg:px-8">
        {/* HEADER */}
        <motion.header
          initial={{ opacity: 0, y: -10 }}
          animate={{ opacity: 1, y: 0 }}
          className="flex items-center justify-between gap-4"
        >
          <Link
            href="/"
            className="inline-flex items-center gap-3 rounded-full border border-slate-200 bg-white/70 px-4 py-2 text-sm font-semibold text-foreground shadow-sm backdrop-blur dark:bg-slate-900/70 dark:border-slate-800"
          >
            <Image
              src="/logo.png"
              alt="Portal Logo"
              width={26}
              height={26}
              priority
              className="rounded-lg bg-foreground dark:bg-background"
            />
            Portal
          </Link>

          <div className="flex items-center gap-2">
            <Link
              className="rounded-full border border-slate-300 px-4 py-2 text-sm transition hover:border-slate-400 hover:bg-white dark:border-slate-700 dark:hover:bg-slate-800"
              href="https://github.com/Spectra010s/portal"
              target="_blank"
            >
              GitHub
            </Link>

            <Link
              className="rounded-full bg-background px-4 py-2 text-sm font-semibold text-foreground transition hover:opacity-90"
              href="/docs"
            >
              Read Docs
            </Link>
          </div>
        </motion.header>

        {/* HERO */}
        <section className="grid items-center gap-10 py-12 md:gap-16 md:py-20 lg:grid-cols-2">
          <motion.div
            initial="initial"
            animate="animate"
            variants={staggerContainer}
            className="min-w-0"
          >
            <motion.p
              variants={fadeInUp}
              className="inline-flex items-center gap-2 rounded-full border border-primary/40 bg-primary/10 px-3 py-1 text-[11px] font-semibold uppercase tracking-[0.18em] text-primary"
            >
              <span className="h-1.5 w-1.5 rounded-full bg-secondary animate-pulse" />
              v0.10.1 is live
            </motion.p>

            <motion.h1
              variants={fadeInUp}
              className="mt-6 text-4xl font-bold tracking-[-0.06em] leading-[0.95] md:text-6xl lg:text-7xl"
            >
              Transfer anything, anywhere, on your own terms.
            </motion.h1>

            <motion.p
              variants={fadeInUp}
              className="mt-6 max-w-xl text-base md:text-lg leading-7 text-slate-600 dark:text-slate-400"
            >
              A lightweight CLI tool to transfer items between devices locally
              or remotely. No middleman, no cloud limits, just raw speed.
            </motion.p>

            <motion.div
              variants={fadeInUp}
              className="mt-8 flex flex-col gap-3 sm:flex-row sm:flex-wrap"
            >
              <Link
                className="inline-flex h-12 w-full sm:w-auto items-center justify-center rounded-2xl bg-primary px-6 text-sm font-semibold !text-white shadow-lg transition hover:-translate-y-0.5 hover:opacity-90"
                href="/docs/install"
              >
                Install Portal
              </Link>

              <Link
                className="inline-flex h-12 w-full sm:w-auto items-center justify-center rounded-2xl border border-slate-300 bg-background/80 px-6 text-sm font-semibold text-foreground transition hover:border-slate-400 dark:border-slate-700"
                href="/docs/usage"
              >
                Explore Usage
              </Link>
            </motion.div>
          </motion.div>

          {/* TERMINAL */}
          <motion.div
            initial={{ opacity: 0, scale: 0.95, x: 20 }}
            animate={{ opacity: 1, scale: 1, x: 0 }}
            transition={{ delay: 0.2, duration: 0.8, ease: [0.19, 1, 0.22, 1] }}
            className="rounded-[2rem] border border-slate-200 bg-background/60 p-4 shadow-2xl backdrop-blur dark:border-slate-800 md:p-6 w-full max-w-full overflow-hidden"
          >
            <div className="mb-5 flex items-center justify-between">
              <p className="text-[11px] font-semibold uppercase tracking-[0.24em] text-foreground">
                Quick Install
              </p>
              <Image
                src="/logo.png"
                alt="Portal"
                width={32}
                height={32}
                className="rounded-xl opacity-50"
              />
            </div>
            <InstallTerminal />
          </motion.div>
        </section>

        {/* HIGHLIGHTS */}
        <motion.section
          initial="initial"
          whileInView="animate"
          viewport={{ once: true, margin: "-100px" }}
          variants={staggerContainer}
          className="grid gap-6 border-t border-slate-200/80 py-14 md:grid-cols-3 dark:border-slate-800"
        >
          <motion.div
            variants={fadeInUp}
            className="mb-10 max-w-3xl md:col-span-3"
          >
            <p className="text-[11px] font-semibold uppercase tracking-[0.24em] text-secondary">
              Features
            </p>
            <h2 className="mt-3 text-3xl font-semibold tracking-[-0.05em] text-foreground md:text-4xl">
              Built for direct, high-speed transfer workflows.
            </h2>
            <p className="mt-4 text-base md:text-lg leading-7 text-slate-600 dark:text-slate-400">
              Portal provides a focused set of capabilities for peer-to-peer
              data movement across local and remote environments.
            </p>
          </motion.div>

          <div className="grid gap-6 md:grid-cols-3 md:col-span-3">
            {highlights.map((item) => (
              <motion.div
                key={item.title}
                variants={fadeInUp}
                whileHover={{ y: -5 }}
                className="rounded-3xl border border-slate-200 bg-background/30 p-6 shadow-sm backdrop-blur dark:border-slate-800"
              >
                <div className="mb-4 flex h-10 w-10 items-center justify-center rounded-2xl bg-primary/10">
                  <item.icon className="h-5 w-5 text-primary" />
                </div>
                <h2 className="text-base font-semibold tracking-[-0.03em] text-foreground">
                  {item.title}
                </h2>
                <p className="mt-2 text-sm md:text-base leading-6 text-slate-600 dark:text-slate-400">
                  {item.desc}
                </p>
              </motion.div>
            ))}
          </div>
        </motion.section>

        {/* Documentation */}
        <motion.section
          initial="initial"
          whileInView="animate"
          viewport={{ once: true }}
          variants={staggerContainer}
          className="grid gap-10 border-t border-slate-200/80 py-14 xl:grid-cols-[1.5fr_1fr] dark:border-slate-800"
        >
          <motion.div variants={fadeInUp}>
            <p className="text-[11px] font-semibold uppercase tracking-[0.24em] text-secondary">
              Documentation
            </p>
            <h2 className="mt-3 text-3xl font-semibold tracking-[-0.05em] text-foreground md:text-4xl">
              Engineered for low-latency, peer-to-peer transport.
            </h2>
            <p className="mt-4 max-w-3xl text-base md:text-lg leading-7 text-slate-600 dark:text-slate-400">
              Portal handles the heavy lifting of network discovery and
              encrypted bitstream delivery.
            </p>
            <div className="mt-8 grid gap-4 md:grid-cols-2">
              {[
                {
                  title: "Installation",
                  desc: "Binary setup for Linux, macOS, Windows, and Termux.",
                  href: "/docs/install",
                },
                {
                  title: "Identity & Verification",
                  desc: "How Portal handles peer discovery and security.",
                  href: "/docs/security",
                },
              ].map((link) => (
                <Link
                  key={link.title}
                  href={link.href}
                  className="rounded-3xl border border-slate-200 bg-background/60 px-5 py-5 transition hover:border-slate-300 dark:border-slate-800 dark:bg-slate-900/40"
                >
                  <h3 className="text-base font-semibold text-foreground">
                    {link.title}
                  </h3>
                  <p className="mt-2 text-sm md:text-base leading-6 text-slate-600 dark:text-slate-400">
                    {link.desc}
                  </p>
                </Link>
              ))}
            </div>
          </motion.div>

          <motion.div
            variants={fadeInUp}
            className="w-full rounded-[2rem] border border-slate-200 bg-background/60 p-10 text-foreground shadow-sm dark:border-slate-800 dark:bg-slate-900/50"
          >
            {" "}
            <p className="text-[11px] font-semibold uppercase tracking-[0.24em] text-foreground">
              Current Status
            </p>
            <ul className="mt-5 justify-between flex flex-1 flex-col space-y-4 text-sm md:text-base leading-6 text-slate-600 dark:text-slate-400">
              <li>• Native CLI transport is the primary production path.</li>
              <li>• Full Termux compatibility for mobile terminal usage.</li>
              <li>• Continuous audits on local-first security protocols.</li>
            </ul>
          </motion.div>
        </motion.section>

        {/* CTA */}
        <motion.section
          initial={{ opacity: 0, y: 30 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          className="py-14"
        >
          <div className="flex flex-col gap-8 rounded-[2rem] dark:border-slate-800 dark:bg-slate-900 px-6 py-10 shadow-2xl md:px-12 md:py-16">
            <div className="max-w-3xl">
              <h2 className="text-3xl font-semibold md:text-5xl">
                Ready to dive into the details?
              </h2>
              <p className="mt-4 opacity-70 md:text-lg">
                Explore the full documentation for setup guides and advanced CLI
                features.
              </p>
            </div>
            <Link
              className="inline-flex h-12 w-full sm:w-auto items-center justify-center rounded-full px-8 text-sm font-bold bg-slate-900 !text-white dark:bg-slate-500 !dark:text-slate-600 transition hover:opacity-90 shadow-md"
              href="/docs"
            >
              Open Documentation
            </Link>
          </div>
        </motion.section>
      </main>
      <Footer />
    </div>
  );
}
