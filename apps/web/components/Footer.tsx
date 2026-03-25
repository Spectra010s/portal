import Link from "next/link";
import Image from "next/image";
import ThemeToggler from "./ThemeToggler";

export default function Footer() {
  return (
    <footer className="w-full border-t border-slate-200/80 px-4 py-10 dark:border-slate-800 max-w-8xl">
      <div className="flex flex-col items-start justify-between gap-10 lg:flex-row lg:items-start">
        {/* Brand Section */}
        <div className="flex flex-col gap-4 w-full lg:w-auto">
          <div className="flex items-center justify-between lg:justify-start lg:gap-6">
            <div className="flex items-center gap-3">
              <Image
                src="/icon.png"
                alt="Portal"
                width={24}
                height={24}
                className="rounded-md opacity-80"
              />
              <span className="text-sm font-bold tracking-tight text-foreground">
                Hiverra Portal
              </span>
            </div>

            <ThemeToggler />
          </div>
          <p className="max-w-xs text-xs leading-5 text-slate-500">
            High-speed, peer-to-peer transport layer for the Hiverra ecosystem.
            Built for performance and technical sovereignty.
          </p>
        </div>

        {/* Technical Specs Grid */}
        <div className="flex flex-wrap gap-8 md:gap-16">
          <div className="flex flex-col gap-2">
            <p className="text-[10px] font-bold uppercase tracking-[0.2em] text-secondary">
              Architect
            </p>
            <a
              href="https://github.com/Spectra010s"
              target="_blank"
              className="text-sm font-medium text-foreground transition hover:text-primary"
            >
              @Spectra010s
            </a>
          </div>
          <div className="flex flex-col gap-2">
            <p className="text-[10px] font-bold uppercase tracking-[0.2em] text-secondary">
              Systems
            </p>
            <div className="flex items-center gap-2">
              <span className="h-1.5 w-1.5 rounded-full bg-emerald-500 animate-pulse" />
              <p className="text-sm font-medium text-slate-600 dark:text-slate-400">
                Operational
              </p>
            </div>
          </div>
          <div className="flex flex-col gap-2">
            <p className="text-[10px] font-bold uppercase tracking-[0.2em] text-secondary">
              Release
            </p>
            <p className="text-sm font-mono font-medium text-slate-600 dark:text-slate-400">
              v0.10.1
            </p>
          </div>
        </div>
      </div>

      {/* Bottom Bar */}
      <div className="mt-10 flex flex-row items-center justify-between border-t border-slate-100 pt-8 dark:border-slate-800/50">
        <p className="text-[10px] text-slate-400 sm:text-[11px]">
          © {new Date().getFullYear()} Hiverra
        </p>
        <div className="flex items-center gap-3 sm:gap-6">
          <Link
            href="/privacy"
            className="text-[10px] text-slate-400 sm:text-[11px]"
          >
            Privacy
          </Link>
          <Link
            href="/terms"
            className="text-[10px] text-slate-400 sm:text-[11px]"
          >
            Terms
          </Link>
          <Link
            href="https://github.com/Spectra010s/portal"
            className="text-[10px] text-slate-400 sm:text-[11px]"
          >
            Source
          </Link>
        </div>
      </div>
    </footer>
  );
}
