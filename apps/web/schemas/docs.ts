import type { TechArticleNode } from "./interface";

export const generateTechArticleNode = (input: {
  title: string;
  description?: string;
  url: string;
}): TechArticleNode => {
  const siteUrl = process.env.NEXT_PUBLIC_SITE_URL || "https://portal.biuld.app";
  const absUrl = input.url.startsWith("http") ? input.url : `${siteUrl}${input.url.startsWith("/") ? input.url : `/${input.url}`}`;
  return {
    "@type": "TechArticle",
    "@id": `${absUrl}#article`,
    headline: input.title,
    description: input.description ?? input.title,
    url: absUrl,
    image: { "@type": "ImageObject", url: `${siteUrl}/opengraph-image` },
    author: { "@id": `${siteUrl}/#organization` },
    publisher: { "@id": `${siteUrl}/#organization` },
    isPartOf: { "@id": `${siteUrl}/#website` },
    mainEntityOfPage: { "@type": "WebPage", "@id": absUrl },
  };
};