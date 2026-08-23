import type { BreadcrumbListNode } from "@/schemas/interface";

const siteUrl = process.env.NEXT_PUBLIC_SITE_URL || "https://portal.biuld.app";

const toAbsoluteUrl = (path: string) => {
  if (path.startsWith("http://") || path.startsWith("https://")) return path;
  return `${siteUrl}${path.startsWith("/") ? path : `/${path}`}`;
};

type BreadcrumbItem = { name: string; path: string };

export const generateBreadcrumbNode = (items: BreadcrumbItem[]): BreadcrumbListNode => {
  const fullItems: BreadcrumbItem[] = [{ name: "Home", path: "/" }, ...items];
  return {
    "@type": "BreadcrumbList",
    itemListElement: fullItems.map((item, index) => ({
      "@type": "ListItem",
      position: index + 1,
      name: item.name,
      item: toAbsoluteUrl(item.path),
    })),
  };
};