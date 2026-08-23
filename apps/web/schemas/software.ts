import type { SoftwareApplicationNode } from "./interface";
import { creator } from "./creator";

const siteUrl = process.env.NEXT_PUBLIC_SITE_URL || "https://portal.biuld.app";

export const softwareNode: SoftwareApplicationNode = {
  "@type": "SoftwareApplication",
  "@id": `${siteUrl}/#software`,
  name: "Hiverra Portal",
  applicationCategory: "UtilitiesApplication",
  operatingSystem: "Windows, macOS, Linux, Android",
  url: siteUrl,
  description: "Best File Transfer Tool — a lightweight CLI to transfer files between devices locally or remotely.",
  author: creator,
  creator,
  codeRepository: "https://github.com/Spectra010s/portal",
  downloadUrl: "https://github.com/Spectra010s/portal/releases",
  sameAs: ["https://github.com/Spectra010s/portal", "https://www.npmjs.com/package/@hiverra/portal", "https://portal.biuld.app"],
  publisher: { "@id": `${siteUrl}/#organization` },
};