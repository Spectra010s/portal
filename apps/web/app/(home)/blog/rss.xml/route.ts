import { Feed } from "feed";
import { blogLoader } from "@/lib/source";
import { NextResponse } from "next/server";

export const revalidate = false;

const siteUrl = process.env.NEXT_PUBLIC_SITE_URL || "https://portal.biuld.app";

export function GET() {
  const feed = new Feed({
    title: "Portal Blog",
    id: `${siteUrl}/blog`,
    link: `${siteUrl}/blog`,
    language: "en",
    favicon: `${siteUrl}/icon.png`,
    copyright: "All rights reserved 2026, Hiverra Portal",
  });

  for (const page of [...blogLoader.getPages()].sort(
    (a, b) =>
      new Date(b.data.date ?? "").getTime() -
      new Date(a.data.date ?? "").getTime()
  )) {
    feed.addItem({
      id: page.url,
      title: page.data.title,
      description: page.data.description,
      link: `${siteUrl}${page.url}`,
      date: new Date(page.data.date ?? ""),
      author: [
        {
          name: page.data.author,
        },
      ],
    });
  }

  return new NextResponse(feed.rss2());
}