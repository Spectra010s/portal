import { DocsLayout } from "fumadocs-ui/layouts/docs";
import type { ReactNode } from "react";
import type { Metadata } from "next";
import { source } from "@/lib/source";
import Image from "next/image";

const siteUrl = process.env.NEXT_PUBLIC_SITE_URL || "https://portal.biuld.app";

export const metadata: Metadata = {
  metadataBase: new URL(siteUrl),
  title: {
    default: "Portal Docs",
    template: "%s | Portal Docs",
  },
  description:
    "Portal documentation for installation, transfer workflows, and troubleshooting.",
  alternates: {
    canonical: "/docs",
  },
  openGraph: {
    type: "website",
    url: "/docs",
    title: "Portal Docs",
    description:
      "Portal documentation for installation, transfer workflows, and troubleshooting.",
    siteName: "Portal",
    images: [
      {
        url: "/docs/opengraph-image",
        width: 1200,
        height: 630,
        alt: "Portal Docs",
      },
    ],
  },
  twitter: {
    card: "summary_large_image",
    title: "Portal Docs",
    description:
      "Portal documentation for installation, transfer workflows, and troubleshooting.",
    images: ["/docs/opengraph-image"],
  },
};

export default function Layout({ children }: { children: ReactNode }) {
  return (
    <DocsLayout
      tree={source.pageTree}
      nav={{
        title: (
          <div className="flex items-center gap-1.5">
            <Image
              src="/logo.png"
              alt="Portal Logo"
              width={28}
              height={28}
              priority
              className="rounded-lg"
            />
            <span className="font-semibold text-foreground">Portal Docs</span>
          </div>
        ),
      }}
      links={[
        {
          type: "icon",
          url: "https://github.com/Spectra010s/portal",
          label: "GitHub",
          text: "GitHub",
          icon: (
            <svg role="img" viewBox="0 0 24 24" fill="currentColor" className="size-5">
              <path d="M12 .297c-6.63 0-12 5.373-12 12 0 5.303 3.438 9.8 8.205 11.385.6.113.82-.258.82-.577 0-.285-.01-1.04-.015-2.04-3.338.724-4.042-1.61-4.042-1.61C4.422 18.07 3.633 17.7 3.633 17.7c-1.087-.744.084-.729.084-.729 1.205.084 1.838 1.236 1.838 1.236 1.07 1.835 2.809 1.305 3.495.998.108-.776.417-1.305.76-1.605-2.665-.3-5.466-1.332-5.466-5.93 0-1.31.465-2.38 1.235-3.22-.135-.303-.54-1.523.105-3.176 0 0 1.005-.322 3.3 1.23.96-.267 1.98-.399 3-.405 1.02.006 2.04.138 3 .405 2.28-1.552 3.285-1.23 3.285-1.23.645 1.653.24 2.873.12 3.176.765.84 1.23 1.91 1.23 3.22 0 4.61-2.805 5.625-5.475 5.92.42.36.81 1.096.81 2.22 0 1.606-.015 2.896-.015 3.286 0 .315.21.69.825.57C20.565 22.092 24 17.592 24 12.297c0-6.627-5.373-12-12-12" />
            </svg>
          ),
          external: true,
        },
      ]}
    >
      {children}
    </DocsLayout>
  );
}
