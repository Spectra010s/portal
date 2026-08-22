import type { Metadata } from "next";
import { notFound } from "next/navigation";
import Link from "next/link";
import { blogLoader } from "@/lib/source";
import { DocsBody } from "fumadocs-ui/page";
import defaultMdxComponents from "fumadocs-ui/mdx";

type PageProps = {
  params: Promise<{ slug: string }>;
};

export default async function BlogPostPage({ params }: PageProps) {
  const { slug } = await params;
  const page = blogLoader.getPage([slug]);

  if (!page) notFound();

  const MDX = page.data.body;

  return (
    <article className="mx-auto w-full max-w-3xl flex-1 px-6 py-12 md:py-16">
      <p className="text-xs font-medium text-slate-500 dark:text-slate-400">
        {new Date(page.data.date ?? "").toDateString()}
      </p>
      <h1 className="mt-3 text-3xl font-semibold tracking-[-0.05em] text-foreground md:text-4xl">
        {page.data.title}
      </h1>
      <p className="mt-4 text-base md:text-lg leading-7 text-slate-600 dark:text-slate-400">
        {page.data.description}
      </p>

      <DocsBody className="mt-10">
        <MDX components={defaultMdxComponents} />
      </DocsBody>

      <div className="mt-12 border-t border-slate-200 pt-6 dark:border-slate-800">
        <Link
          href="/blog"
          className="text-sm font-medium text-primary hover:underline"
        >
          Back to blog
        </Link>
      </div>
    </article>
  );
}

export function generateStaticParams() {
  return blogLoader.getPages().map((page) => ({
    slug: page.slugs[0],
  }));
}

export async function generateMetadata({ params }: PageProps): Promise<Metadata> {
  const { slug } = await params;
  const page = blogLoader.getPage([slug]);

  if (!page) notFound();

  return {
    title: page.data.title,
    description: page.data.description,
  };
}