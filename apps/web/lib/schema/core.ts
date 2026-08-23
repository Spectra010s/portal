export type SchemaNode = {
  "@type": string;
  "@id"?: string;
};

export type SchemaGraph<T extends SchemaNode = SchemaNode> = {
  "@context": "https://schema.org";
  "@graph": T[];
};

export const createSchema = <T extends SchemaNode>(nodes: T[]): SchemaGraph<T> => ({
  "@context": "https://schema.org",
  "@graph": nodes,
});