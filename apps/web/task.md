# Docs Metadata & Nextra Updates

## Phase Plan

- [x] Phase 1: Add per-page `title` and `description` metadata to docs MDX pages.
- [x] Phase 2: Improve docs layout metadata in `app/docs/layout.tsx` (`metadataBase`, Open Graph, Twitter, canonical).
- [x] Phase 3: Open Graph rollout.
- [ ] Phase 4: Tune Nextra layout config (navbar links, edit/feedback labels, sidebar/toc behavior).

## Phase 1 Notes

- Source for descriptions: `lib/docs.ts` (`docsLinks`).
- Overview title set to: `About Portal`.
- Updated files:
  - `content/overview.mdx`
  - `content/install.mdx`
  - `content/usage.mdx`
  - `content/cli-cli.mdx`
  - `content/troubleshooting.mdx`
  - `content/faq.mdx`

## Phase 3: Open Graph Rollout

- [x] 3.1 Refresh root share card in `app/opengraph-image.tsx`.
- [x] 3.2 Add docs segment share card in `app/docs/opengraph-image.tsx`.
- [x] 3.3 Wire docs layout metadata image fields to the docs OG route.
- [x] 3.4 Add selective per-page OG metadata for high-value docs pages only:
  - `/docs/install`
  - `/docs/usage`
  - `/docs/cli-cli`
- [x] 3.5 Keep all other docs pages on the docs-segment OG default to reduce maintenance.

## Open Questions

- Confirm final OG visual direction for root (`app/*`) vs docs (`app/docs/*`) so both are brand-consistent but distinct.
