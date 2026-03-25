import type { Metadata, Viewport } from "next";
import "./globals.css";
import { ThemeProvider } from "next-themes";
import Footer from "@/components/Footer";

const siteUrl = process.env.NEXT_PUBLIC_SITE_URL || "https://portal.biuld.app";

export const metadata: Metadata = {
  metadataBase: new URL(siteUrl),
  title: {
    default: "Hiverra Portal",
    template: "%s | Portal",
  },
  description:
    "Portal: A lightweight CLI tool to transfer files between devices locally or remotely.",
  manifest: "/manifest.json",
  alternates: {
    canonical: "/",
  },
  icons: [
    {
      rel: "icon",
      type: "image/png",
      sizes: "16x16",
      url: "/favicon-16x16.png",
    },
    {
      rel: "icon",
      type: "image/png",
      sizes: "32x32",
      url: "/favicon-32x32.png",
    },
    {
      rel: "icon",
      type: "image/png",
      sizes: "500x500",
      url: "/icon.png",
    },
    {
      rel: "apple-touch-icon",
      url: "/apple-touch-icon.png",
    },
  ],
  openGraph: {
    type: "website",
    title: "Hiverra Portal",
    description:
      "Portal: A lightweight CLI tool to transfer files between devices locally or remotely.",
    siteName: "Portal",
    images: [
      {
        url: "/opengraph-image",
        width: 1200,
        height: 630,
        alt: "Portal",
      },
    ],
  },
  twitter: {
    card: "summary_large_image",
    title: "Hiverra Portal",
    description:
      "Portal: A lightweight CLI tool to transfer files between devices locally or remotely.",
    images: ["/opengraph-image"],
    creator: "@Spectra010s",
    site: "@Spectra010s",
  },
};

export const viewport: Viewport = {
  themeColor: "#0369a1",
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html
      lang="en"
      className="h-full antialiased"
      dir="ltr"
      suppressHydrationWarning
    >
      <body>
        <ThemeProvider attribute="class" defaultTheme="system" enableSystem>
          {children}
        </ThemeProvider>
      </body>
    </html>
  );
}
