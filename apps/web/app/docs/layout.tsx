import type { Metadata } from "next";
import type { ReactNode } from "react";
import { Layout, Navbar } from "nextra-theme-docs";
import { Banner } from "nextra/components";
import { getPageMap } from "nextra/page-map";
import "nextra-theme-docs/style.css";
import Footer from "@/components/Footer";

const siteUrl = process.env.NEXT_PUBLIC_SITE_URL;

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

const navbar = (
  <Navbar
    logo={
      <div className="flex items-center">
        <img
          src="/icon.png"
          alt="Logo"
          width="24"
          height="24"
          className="rounded-lg"
        />
        <span className="ml-[0.4em] font-extrabold text-foreground">
          Portal Docs
        </span>
      </div>
    }
    projectLink="https://github.com/Spectra010s/portal"
  />
);

const banner = (
  <Banner storageKey="portal-0.10.1-release">
    <a
      href="https://github.com/Spectra010s/portal/releases/latest"
      target="_blank"
    >
      🎉 Portal 0.10.1 is released. Read more →
    </a>
  </Banner>
);

export default async function DocsLayout({
  children,
}: {
  children: ReactNode;
}) {
  const pageMap = await getPageMap("/docs");

  return (
    <Layout
      banner={banner}
      navbar={navbar}
      footer={<Footer />}
      docsRepositoryBase="https://github.com/Spectra010s/portal/tree/main/apps/web"
      pageMap={pageMap}
      editLink="Edit this page on GitHub"
      sidebar={{
        defaultMenuCollapseLevel: 2,
        toggleButton: true,
      }}
      feedback={{ content: "Question? Give feedback", labels: "documentation" }}
    >
      {children}
    </Layout>
  );
}
