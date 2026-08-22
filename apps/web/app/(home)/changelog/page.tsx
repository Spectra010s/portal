import ReactMarkdown from "react-markdown";
import remarkGfm from "remark-gfm";
import Link from "next/link";

type Release = {
  tag_name: string;
  name: string;
  published_at: string;
  html_url: string;
  body: string | null;
};

async function getReleases(): Promise<Release[]> {
  try {
    const res = await fetch(
      "https://api.github.com/repos/Spectra010s/portal/releases",
      { next: { revalidate: 3600 } }
    );
    if (!res.ok) return [];
    return res.json();
  } catch {
    return [];
  }
}

function formatMonth(d: Date): string {
  return d.toLocaleDateString("en-US", { month: "long", year: "numeric", timeZone: "UTC" });
}
function formatDay(d: Date): string {
  return d.toLocaleDateString("en-US", { month: "short", day: "numeric", timeZone: "UTC" });
}
function groupByMonth(releases: Release[]): [string, Release[]][] {
  const map = new Map<string, Release[]>();
  for (const r of releases) {
    const key = formatMonth(new Date(r.published_at));
    const list = map.get(key);
    if (list) list.push(r);
    else map.set(key, [r]);
  }
  return Array.from(map.entries());
}

export default async function ChangelogPage() {
  const releases = await getReleases();
  const grouped = groupByMonth(releases);

  return (
    <main className="mx-auto w-full max-w-3xl flex-1 px-6 py-12 md:py-16">
      <p className="text-[11px] font-semibold uppercase tracking-[0.24em] text-secondary">Changelog</p>
      <h1 className="mt-3 text-3xl font-semibold tracking-[-0.05em] text-foreground md:text-4xl">Release history</h1>
      <p className="mt-4 text-base md:text-lg leading-7 text-slate-600 dark:text-slate-400">Every release, in order. Latest at the top.</p>

      {releases.length === 0 ? (
        <p className="mt-10 text-sm text-slate-500 dark:text-slate-400">
          Couldn&apos;t load releases. Try refreshing, or view them on{" "}
          <a href="https://github.com/Spectra010s/portal/releases" target="_blank" rel="noreferrer" className="text-primary underline-offset-4 hover:underline">
            GitHub
          </a>
          .
        </p>
      ) : (
        <div className="mt-10 flex flex-col gap-8">
          {grouped.map(([month, items]) => (
          <section key={month} className="relative">
            <h2 className="sticky top-0 z-10 bg-background/80 py-4 text-sm font-medium text-slate-500 backdrop-blur-sm dark:text-slate-400">{month}</h2>
            <ol className="relative ms-3 border-s border-slate-200 ps-6 dark:border-slate-800">
              {items.map((release) => (
                <li key={release.tag_name} className="relative pb-10 last:pb-0">
                  <span className="absolute -start-6 translate-x-[calc(-50%-0.5px)] top-1.5 size-2.5 rounded-full border-2 border-primary bg-background" />
                  <div className="flex flex-wrap items-center gap-1.5 mb-4">
                    <time dateTime={release.published_at} className="text-xs font-medium tabular-nums text-primary pe-2">
                      {formatDay(new Date(release.published_at))}
                    </time>
                    <Link href={release.html_url} target="_blank" rel="noreferrer" className="rounded-md border border-slate-200 bg-slate-100 px-1.5 py-0.5 font-mono text-xs text-slate-700 hover:underline dark:border-slate-700 dark:bg-slate-800 dark:text-slate-300">
                      {release.tag_name}
                    </Link>
                  </div>
                  {release.body && (
                    <div className="prose prose-sm dark:prose-invert max-w-none text-sm leading-6 prose-a:text-primary prose-a:underline-offset-4 hover:prose-a:underline prose-h2:text-lg prose-h2:mb-3 prose-h3:text-base prose-h3:mb-2 prose-pre:overflow-x-auto">
                      <ReactMarkdown
                        remarkPlugins={[remarkGfm]}
                        components={{
                          a: (props) => <a {...props} target="_blank" rel="noreferrer" />,
                        }}
                      >
                        {release.body.replace(/(?<!\]\()(?<!\[)https?:\/\/[^\s)]+(?![^\[]*\])/g, (url) => `[${url}](${url})`)}
                      </ReactMarkdown>
                    </div>
                  )}
                </li>
              ))}
            </ol>
          </section>
        ))}
        </div>
      )}
    </main>
  );
}