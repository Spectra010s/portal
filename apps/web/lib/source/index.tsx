import { loader } from "fumadocs-core/source";
import { defineCollections, defineDocs } from "fumadocs-mdx/macro";
import { pageSchema } from "fumadocs-core/source/schema";
import { z } from "zod";

const docs = defineDocs({
  dir: "content/docs",
  docs: {
    postprocess: {
      includeProcessedMarkdown: true,
    },
  },
});

const blog = defineCollections({
  type: "doc",
  dir: "content/blog",
  schema: pageSchema.extend({
    author: z.string(),
    date: z.iso.date().or(z.date()),
  }),
});

export const source = loader(
  {
    docs: docs.toFumadocsSource(),
  },
  {
    baseUrl: "/docs",
  }
);

export const blogLoader = loader(blog.toFumadocsSource(), {
  baseUrl: "/blog",
});

export type Page = (typeof source)["$inferPage"];
export type Meta = (typeof source)["$inferMeta"];