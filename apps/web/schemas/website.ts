import type { WebSiteNode } from "./interface";

const siteUrl = process.env.NEXT_PUBLIC_SITE_URL || "https://portal.biuld.app";

export const websiteNode: WebSiteNode = {
  "@type": "WebSite",
  "@id": `${siteUrl}/#website`,
  url: siteUrl,
  name: "Hiverra Portal",
  alternateName: "Portal",
  publisher: { "@id": `${siteUrl}/#organization` },
  potentialAction: {
    "@type": "SearchAction",
    target: `${siteUrl}/search?q={search_term_string}`,
    "query-input": "required name=search_term_string",
  },
};