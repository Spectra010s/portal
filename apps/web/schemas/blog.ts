import type { BlogPostingNode } from "./interface";

const siteUrl = process.env.NEXT_PUBLIC_SITE_URL || "https://portal.biuld.app";

export const generateBlogPostingNode = (input: {
  slug: string;
  title: string;
  description: string;
  author: string;
  date: string | Date;
}): BlogPostingNode => {
  const url = `${siteUrl}/blog/${input.slug}`;
  const iso = new Date(input.date).toISOString();
  return {
    "@type": "BlogPosting",
    "@id": `${url}#article`,
    headline: input.title,
    description: input.description,
    url,
    image: { "@type": "ImageObject", url: `${siteUrl}/opengraph-image` },
    author: { "@type": "Person", name: input.author },
    publisher: { "@id": `${siteUrl}/#organization` },
    datePublished: iso,
    dateModified: iso,
    isPartOf: { "@id": `${siteUrl}/#website` },
    mainEntityOfPage: { "@type": "WebPage", "@id": url },
  };
};