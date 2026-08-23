export interface BreadcrumbListNode {
  "@type": "BreadcrumbList";
  itemListElement: Array<{
    "@type": "ListItem";
    position: number;
    name: string;
    item: string;
  }>;
}

export interface WebSiteNode {
  "@type": "WebSite";
  "@id": string;
  url: string;
  name: string;
  alternateName?: string;
  description?: string;
  publisher: { "@id": string };
  creator?: {
    "@type": "Person";
    name: string;
    url: string;
  };
  potentialAction: {
    "@type": "SearchAction";
    target: string;
    "query-input": string;
  };
}

export interface SoftwareApplicationNode {
  "@type": "SoftwareApplication";
  "@id": string;
  name: string;
  applicationCategory: string;
  operatingSystem: string;
  url: string;
  description: string;
  publisher: { "@id": string };
  author?: { "@type": "Person"; name: string; url: string };
  creator?: { "@type": "Person"; name: string; url: string };
  codeRepository?: string;
  downloadUrl?: string;
  sameAs?: string[];
}

export interface BlogPostingNode {
  "@type": "BlogPosting";
  "@id": string;
  headline: string;
  description: string;
  url: string;
  image: { "@type": "ImageObject"; url: string };
  author: { "@type": "Person"; name: string } | { "@id": string };
  publisher: { "@id": string };
  datePublished: string;
  dateModified: string;
  isPartOf: { "@id": string };
  mainEntityOfPage: { "@type": "WebPage"; "@id": string };
}

export interface TechArticleNode {
  "@type": "TechArticle";
  "@id": string;
  headline: string;
  description: string;
  url: string;
  image: { "@type": "ImageObject"; url: string };
  author: { "@id": string };
  publisher: { "@id": string };
  isPartOf: { "@id": string };
  mainEntityOfPage: { "@type": "WebPage"; "@id": string };
}