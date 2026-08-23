import type { WebSiteNode } from "./interface";
import { creator } from "./creator";

const siteUrl = process.env.NEXT_PUBLIC_SITE_URL || "https://portal.biuld.app";

export const websiteNode: WebSiteNode = {
  "@type": "WebSite",
  "@id": `${siteUrl}/#website`,
  url: siteUrl,
  name: "Hiverra Portal",
  alternateName: "Portal",
  description: "Best File Transfer Tool — a lightweight CLI to transfer files between devices locally or remotely.",
  creator,
  publisher: { "@id": `${siteUrl}/#organization` },
  potentialAction: {
    "@type": "SearchAction",
    target: `${siteUrl}/search?q={search_term_string}`,
    "query-input": "required name=search_term_string",
  },
};