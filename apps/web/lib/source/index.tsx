import { loader } from "fumadocs-core/source";
import { docs } from "collections/server";

export const source = loader(
  {
    docs: docs.toFumadocsSource(),
  },
  {
    baseUrl: "/docs",
  }
);

export type Page = (typeof source)["$inferPage"];
export type Meta = (typeof source)["$inferMeta"];
