import type { MetadataRoute } from "next";
import { source, blogLoader } from "@/lib/source";

export default function sitemap(): MetadataRoute.Sitemap {
  const siteUrl = process.env.NEXT_PUBLIC_SITE_URL || "https://portal.biuld.app";

  const docsPages = source.getPages().map((page) => ({
    url: `${siteUrl}${page.url}`,
    lastModified: new Date(),
  }));

  const blogPages = blogLoader.getPages().map((page) => ({
    url: `${siteUrl}${page.url}`,
    lastModified: page.data.date ? new Date(page.data.date) : new Date(),
  }));

  return [
    { url: siteUrl, lastModified: new Date() },
    { url: `${siteUrl}/docs`, lastModified: new Date() },
    { url: `${siteUrl}/blog`, lastModified: new Date() },
    { url: `${siteUrl}/changelog`, lastModified: new Date() },
    ...docsPages,
    ...blogPages,
  ];
}