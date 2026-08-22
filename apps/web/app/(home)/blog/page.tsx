import Link from "next/link";
import { blogLoader } from "@/lib/source";

export default function BlogPage() {
  const posts = [...blogLoader.getPages()].sort(
    (a, b) =>
      new Date(b.data.date ?? "").getTime() -
      new Date(a.data.date ?? "").getTime()
  );

  return (
    <main className="mx-auto w-full max-w-4xl flex-1 px-6 py-12 md:py-16">
      <p className="text-[11px] font-semibold uppercase tracking-[0.24em] text-secondary">
        Blog
      </p>
      <h1 className="mt-3 text-3xl font-semibold tracking-[-0.05em] text-foreground md:text-4xl">
        Latest posts
      </h1>
      <p className="mt-4 text-base md:text-lg leading-7 text-slate-600 dark:text-slate-400">
        Announcements and deep dives into how Portal works.
      </p>

      <div className="mt-10 grid gap-4">
        {posts.map((post) => (
          <Link
            key={post.url}
            href={post.url}
            className="flex flex-col rounded-3xl border border-slate-200 bg-background/60 p-6 transition hover:border-slate-300 dark:border-slate-800 dark:bg-slate-900/40"
          >
            <p className="text-xs font-medium text-slate-500 dark:text-slate-400">
              {new Date(post.data.date ?? "").toDateString()}
            </p>
            <h2 className="mt-2 text-xl font-semibold tracking-[-0.03em] text-foreground">
              {post.data.title}
            </h2>
            <p className="mt-2 text-sm md:text-base leading-6 text-slate-600 dark:text-slate-400">
              {post.data.description}
            </p>
          </Link>
        ))}
      </div>
    </main>
  );
}