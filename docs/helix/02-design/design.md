---
ddx:
  id: weft.microsite-design
  type: design-system
  activity: design
  status: draft
  authoring:
    home: repo
  links:
    - id: weft.vision
      kind: informed_by
    - id: weft.architecture
      kind: informed_by
---

# Weft microsite design

## Navigation and Active State

A single editorial landing page has top navigation to Approach, Backends and Contract, plus a source link. Section links use aria-current="location" when their section is visible; the corresponding link gets a rust underline through the attribute selector. The home link uses aria-current="page" with a bold wordmark. Anchor navigation works without JavaScript.

## Visual Hierarchy

Warm paper, dark ink and a single rust accent establish an engineering publication. A generous two-column hero pairs a large serif proposition with a dark query sheet. A numbered pipeline follows, then backend comparisons and a concrete result contract. Sections are divided by thin rules; no decorative dashboard cards or stock photography. A small woven SVG mark connects the name to intersecting logical and physical paths.

Maximum content width is 1180px. The hero heading uses clamp(3rem, 7vw, 6.8rem), tight line height 0.96; section headings use 2.5rem; body uses 1rem/1.65. Mobile collapses to one column below 760px. Code scrolls within its own region rather than widening the viewport.

## Interaction States

Links underline on hover. Enabled controls use a 3px rust focus-visible outline with 4px offset. Backend tabs use native buttons with aria-selected, a visible border and roving keyboard focus; Left/Right/Home/End select tabs. Content is a labeled tabpanel. The selector changes an explanatory mapping, not a live compiler output; this distinction is visible beside the example. No network-dependent loading, error or disabled state is needed. Reduced motion disables smooth scrolling and transitions.

## Tokens

Color: paper #f4f0e8; surface #fffcf5; ink #202b2a; muted #53605b; rust #943c29; line #c7c9bd; code #182725; code-text #e5ece1. Font: Georgia/serif for display; system sans for body; ui-monospace for code. Space: 4, 8, 12, 16, 24, 32, 48, 72, 112px. Corners are square except 3px on controls. No external fonts or analytics.

## Content and accessibility

One h1, hierarchical h2 headings, skip link, landmarks, descriptive links and text labels for the weave mark. Preserve readable contrast, browser zoom and keyboard navigation. Qualified compiler profiles have fixture evidence; actual storage adoption and package distribution remain separate. Security policy lowering is work in progress. The API example is conceptual and points readers to versioned contracts.

## Non-Goals

This interface system does not define compiler architecture, data flow internals, backend mappings, policy semantics or ADR decisions. Those remain in their governing contracts. It does not provide database execution or a browser compiler playground.
