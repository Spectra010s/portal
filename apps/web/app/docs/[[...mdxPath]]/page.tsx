import { generateStaticParamsFor, importPage } from "nextra/pages";
import { useMDXComponents as getMDXComponents } from "../../../mdx-components";

export const generateStaticParams = generateStaticParamsFor("mdxPath");

export async function generateMetadata(props: {
  params: Promise<{ mdxPath?: string[] }>;
}) {
  const params = await props.params;

  try {
    const { metadata } = await importPage(params.mdxPath);
    const path = params.mdxPath?.join("/") || "";
    const url = path ? "/docs/" + path : "/docs";
    const ogImage = "/docs/opengraph-image";

    return {
      ...metadata,
      openGraph: {
        type: "article",
        url,
        title: metadata?.title,
        description: metadata?.description,
        images: [{ url: ogImage }],
        ...metadata?.openGraph,
      },
      twitter: {
        card: "summary_large_image",
        title: metadata?.title,
        description: metadata?.description,
        images: [ogImage],
        ...metadata?.twitter,
      },
    };
  } catch (e) {
    return { title: "Portal Documentation" };
  }
}

const Wrapper = getMDXComponents({}).wrapper;

export default async function Page(props: {
  params: Promise<{ mdxPath?: string[] }>;
}) {
  const params = await props.params;

  const {
    default: MDXContent,
    toc,
    metadata,
    sourceCode,
  } = await importPage(params.mdxPath);

  return (
    <Wrapper toc={toc} metadata={metadata} sourceCode={sourceCode}>
      <MDXContent {...props} params={params} />
    </Wrapper>
  );
}
