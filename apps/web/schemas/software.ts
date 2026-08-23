import type { SoftwareApplicationNode } from "./interface";

const siteUrl = process.env.NEXT_PUBLIC_SITE_URL || "https://portal.biuld.app";

export const softwareNode: SoftwareApplicationNode = {
  "@type": "SoftwareApplication",
  "@id": `${siteUrl}/#software`,
  name: "Hiverra Portal",
  applicationCategory: "UtilitiesApplication",
  operatingSystem: "Windows, macOS, Linux, Android",
  url: siteUrl,
  description: "Portal: A lightweight CLI tool to transfer files between devices locally or remotely.",
  publisher: { "@id": `${siteUrl}/#organization` },
};