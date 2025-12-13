# Plan: Inline Source Citations for Research Documents

## Problem Statement

Research documents display sources as a simple list of URLs at the bottom. Users cannot see **which statements are backed by which sources**. This makes it hard to:
- Verify specific claims
- Assess the credibility of individual statements
- Quickly access relevant sources for a topic of interest

**Current:** Sources listed at bottom with no connection to content
**Goal:** Claude.ai-style inline citations that show source attribution per statement

---

## Goals

1. **Inline Citation Markers** - Pill-shaped badges like `[1,2]` appear inline with text
2. **Rich Hover Popover** - Shows source titles, site names, and favicons
3. **Click to Open** - Clicking a citation opens the source in a new tab
4. **Summary Labels** - Badge shows topic summary (e.g., "Energy Drink Info") + source count
5. **General Sources** - Non-cited sources still appear at bottom

---

## Visual Reference

```
"Energy drinks contain 160-200mg caffeine [Energy Drink Info +3], well within safe limits."
                                          └──────────┬──────────┘
                                             Pill badge (hoverable)
                                                     │
                                                     ▼ on hover
                                    ┌─────────────────────────────────┐
                                    │ 🌐 Energy Drink Facts | Caff... │
                                    │    energydrinkinfo.com          │
                                    ├─────────────────────────────────┤
                                    │ 🌐 Spilling the Beans: How...   │
                                    │    fda.gov                      │
                                    ├─────────────────────────────────┤
                                    │ 🌐 Safety of caffeine           │
                                    │    europa.eu                    │
                                    └─────────────────────────────────┘
```

---

## Data Model Changes

### New Types

**Citation Reference (in content):**
```
[cite:1,2,3:Energy Drink Info]
```
Format: `[cite:<source_ids>:<summary_label>]`

**Source Info (rich metadata):**
```typescript
interface SourceInfo {
  id: number;           // 1-indexed for citation references
  url: string;
  title: string | null;
  site_name: string | null;
  favicon_url: string | null;
}
```

---

## Implementation Steps

### Step 1: Update Backend Types

**File:** `terminal-research/src/types.rs`

Add `SourceInfo` struct and update `SynthesizedReport`:

```rust
/// Rich source information with metadata for inline citations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceInfo {
    /// 1-indexed ID for citation references in content
    pub id: usize,
    /// The source URL
    pub url: String,
    /// Page title (e.g., "Energy Drink Facts | Caffeine, Ingredients & More")
    pub title: Option<String>,
    /// Site/publisher name (e.g., "American Beverage Association")
    pub site_name: Option<String>,
    /// Favicon URL (can use Google's service as fallback)
    pub favicon_url: Option<String>,
}

impl SourceInfo {
    /// Create from URL with optional metadata
    pub fn new(id: usize, url: String) -> Self {
        let favicon_url = Self::get_favicon_url(&url);
        let site_name = Self::extract_site_name(&url);
        Self {
            id,
            url,
            title: None,
            site_name,
            favicon_url,
        }
    }

    /// Extract domain as site name fallback
    fn extract_site_name(url: &str) -> Option<String> {
        url::Url::parse(url)
            .ok()
            .and_then(|u| u.host_str().map(|h| h.replace("www.", "")))
    }

    /// Get favicon using Google's favicon service
    fn get_favicon_url(url: &str) -> Option<String> {
        url::Url::parse(url)
            .ok()
            .and_then(|u| u.host_str().map(|h| {
                format!("https://www.google.com/s2/favicons?domain={}&sz=32", h)
            }))
    }
}
```

**Update `SynthesizedReport`:**

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SynthesizedReport {
    pub title: String,
    pub executive_summary: String,
    pub sections: Vec<ReportSection>,
    pub key_factors: Vec<KeyFactor>,
    pub confidence_assessment: String,
    /// Rich source info for inline citations (replaces simple Vec<String>)
    pub sources: Vec<SourceInfo>,
    /// Sources that aren't cited inline but are still relevant
    pub general_sources: Vec<String>,
    pub trading_analysis: Option<TradingAnalysis>,
}
```

**Add `url` crate to dependencies in `terminal-research/Cargo.toml`:**
```toml
url = "2"
```

---

### Step 2: Update OpenAI Prompts for Inline Citations

**File:** `terminal-research/src/openai.rs`

Modify the synthesis prompt to instruct the model to output inline citations.

Find the synthesis prompt (likely in a function like `synthesize_report` or similar) and update it:

```rust
// Add to the system prompt for synthesis:
const CITATION_INSTRUCTIONS: &str = r#"
## Citation Format

You MUST include inline citations for factual claims. Use this exact format:
[cite:1,2,3:Topic Summary]

Where:
- Numbers reference source IDs (1-indexed, matching the sources list)
- Topic Summary is a 2-4 word description of what the sources cover
- Multiple source IDs are comma-separated

Examples:
- "The FDA recommends 400mg daily limit [cite:1,2:FDA Guidelines]"
- "Studies show 160-200mg per serving [cite:3,4,5:Caffeine Content]"
- "Market analysts predict growth [cite:7:Market Analysis]"

Rules:
1. Cite specific facts, statistics, dates, and quotes
2. Don't cite common knowledge or your own analysis
3. Group related sources under one citation when they support the same point
4. Use descriptive topic summaries (not generic like "Sources" or "References")
5. Every source in the sources list should be cited at least once if it was used
"#;
```

**Update the source collection to assign IDs:**

When collecting sources during research, ensure each gets a unique ID:

```rust
// When building the sources list:
let sources: Vec<SourceInfo> = collected_urls
    .into_iter()
    .enumerate()
    .map(|(idx, url)| SourceInfo::new(idx + 1, url)) // 1-indexed
    .collect();
```

**Pass source IDs to the synthesis prompt:**

The AI needs to know which ID maps to which source:

```rust
// Format sources for the prompt
fn format_sources_for_prompt(sources: &[SourceInfo]) -> String {
    sources
        .iter()
        .map(|s| format!("[{}] {} - {}", s.id, s.url, s.title.as_deref().unwrap_or("No title")))
        .collect::<Vec<_>>()
        .join("\n")
}

// Include in the synthesis prompt:
let sources_context = format!(
    "## Available Sources (use these IDs in citations)\n{}",
    format_sources_for_prompt(&sources)
);
```

---

### Step 3: Fetch Source Metadata

**File:** `terminal-research/src/openai.rs` or create `terminal-research/src/metadata.rs`

Add a function to fetch page metadata (title) from URLs:

```rust
use scraper::{Html, Selector};

/// Fetch metadata for a source URL
pub async fn fetch_source_metadata(client: &reqwest::Client, url: &str) -> Option<(String, Option<String>)> {
    // Returns (title, site_name)
    let response = client
        .get(url)
        .header("User-Agent", "Mozilla/5.0 (compatible; ResearchBot/1.0)")
        .timeout(std::time::Duration::from_secs(5))
        .send()
        .await
        .ok()?;

    let html = response.text().await.ok()?;
    let document = Html::parse_document(&html);

    // Try to get title
    let title_selector = Selector::parse("title").ok()?;
    let title = document
        .select(&title_selector)
        .next()
        .map(|el| el.text().collect::<String>().trim().to_string());

    // Try to get og:site_name
    let og_selector = Selector::parse(r#"meta[property="og:site_name"]"#).ok()?;
    let site_name = document
        .select(&og_selector)
        .next()
        .and_then(|el| el.value().attr("content").map(String::from));

    title.map(|t| (t, site_name))
}

/// Enrich sources with metadata (run in parallel with timeout)
pub async fn enrich_sources_with_metadata(
    client: &reqwest::Client,
    sources: &mut [SourceInfo],
) {
    use futures::future::join_all;

    let futures: Vec<_> = sources
        .iter()
        .map(|s| fetch_source_metadata(client, &s.url))
        .collect();

    let results = join_all(futures).await;

    for (source, result) in sources.iter_mut().zip(results) {
        if let Some((title, site_name)) = result {
            source.title = Some(title);
            if site_name.is_some() {
                source.site_name = site_name;
            }
        }
    }
}
```

**Add `scraper` to dependencies:**
```toml
scraper = "0.18"
```

**Call metadata enrichment before synthesis:**

```rust
// In the research pipeline, after collecting sources:
enrich_sources_with_metadata(&client, &mut sources).await;
```

---

### Step 4: Update Frontend Types

**File:** `frontend/src/lib/types.ts`

Add `SourceInfo` interface and update `SynthesizedReport`:

```typescript
export interface SourceInfo {
  id: number;
  url: string;
  title: string | null;
  site_name: string | null;
  favicon_url: string | null;
}

export interface SynthesizedReport {
  title: string;
  executive_summary: string;
  sections: ReportSection[];
  key_factors: KeyFactor[];
  confidence_assessment: string;
  sources: SourceInfo[];        // Changed from string[]
  general_sources: string[];    // New field for uncited sources
  trading_analysis?: TradingAnalysis;
}
```

---

### Step 5: Create Citation Components

**File:** `frontend/src/components/research/inline-citation.tsx`

```tsx
"use client";

import { useState, useRef } from "react";
import { SourceInfo } from "@/lib/types";
import { cn } from "@/lib/utils";

interface InlineCitationProps {
  sourceIds: number[];
  label: string;
  sources: SourceInfo[];
}

export function InlineCitation({ sourceIds, label, sources }: InlineCitationProps) {
  const [isOpen, setIsOpen] = useState(false);
  const timeoutRef = useRef<NodeJS.Timeout | null>(null);

  // Get the sources for this citation
  const citedSources = sourceIds
    .map((id) => sources.find((s) => s.id === id))
    .filter((s): s is SourceInfo => s !== undefined);

  const count = citedSources.length;
  const displayLabel = count > 1 ? `${label} +${count - 1}` : label;

  const handleMouseEnter = () => {
    if (timeoutRef.current) clearTimeout(timeoutRef.current);
    setIsOpen(true);
  };

  const handleMouseLeave = () => {
    timeoutRef.current = setTimeout(() => setIsOpen(false), 150);
  };

  const handleClick = () => {
    // Open first source in new tab
    if (citedSources[0]) {
      window.open(citedSources[0].url, "_blank", "noopener,noreferrer");
    }
  };

  return (
    <span className="relative inline-block">
      <button
        onClick={handleClick}
        onMouseEnter={handleMouseEnter}
        onMouseLeave={handleMouseLeave}
        className={cn(
          "inline-flex items-center gap-1 px-2 py-0.5 text-xs rounded-full",
          "bg-muted/50 hover:bg-muted text-muted-foreground hover:text-foreground",
          "border border-border/50 hover:border-border",
          "transition-colors cursor-pointer align-baseline mx-0.5"
        )}
      >
        {displayLabel}
      </button>

      {isOpen && (
        <div
          onMouseEnter={handleMouseEnter}
          onMouseLeave={handleMouseLeave}
          className={cn(
            "absolute z-50 top-full left-0 mt-1 w-72",
            "bg-popover border border-border rounded-lg shadow-lg",
            "animate-in fade-in-0 zoom-in-95 duration-100"
          )}
        >
          <div className="p-1">
            {citedSources.map((source, idx) => (
              <a
                key={source.id}
                href={source.url}
                target="_blank"
                rel="noopener noreferrer"
                className={cn(
                  "flex items-start gap-2 p-2 rounded-md",
                  "hover:bg-muted transition-colors",
                  idx < citedSources.length - 1 && "border-b border-border/50"
                )}
              >
                {source.favicon_url ? (
                  <img
                    src={source.favicon_url}
                    alt=""
                    className="w-4 h-4 mt-0.5 rounded-sm flex-shrink-0"
                    onError={(e) => {
                      (e.target as HTMLImageElement).style.display = "none";
                    }}
                  />
                ) : (
                  <div className="w-4 h-4 mt-0.5 rounded-sm bg-muted flex-shrink-0" />
                )}
                <div className="flex-1 min-w-0">
                  <p className="text-sm font-medium truncate text-foreground">
                    {source.title || source.url}
                  </p>
                  <p className="text-xs text-muted-foreground truncate">
                    {source.site_name || new URL(source.url).hostname}
                  </p>
                </div>
              </a>
            ))}
          </div>
        </div>
      )}
    </span>
  );
}
```

---

### Step 6: Create Citation Parser

**File:** `frontend/src/lib/citation-parser.tsx`

```tsx
import { ReactNode } from "react";
import { SourceInfo } from "./types";
import { InlineCitation } from "@/components/research/inline-citation";

// Regex to match [cite:1,2,3:Label Text]
const CITATION_REGEX = /\[cite:([\d,]+):([^\]]+)\]/g;

interface ParsedSegment {
  type: "text" | "citation";
  content: string;
  sourceIds?: number[];
  label?: string;
}

/**
 * Parse text content and extract citation markers
 */
export function parseCitations(text: string): ParsedSegment[] {
  const segments: ParsedSegment[] = [];
  let lastIndex = 0;

  let match;
  while ((match = CITATION_REGEX.exec(text)) !== null) {
    // Add text before citation
    if (match.index > lastIndex) {
      segments.push({
        type: "text",
        content: text.slice(lastIndex, match.index),
      });
    }

    // Parse citation
    const sourceIds = match[1].split(",").map((id) => parseInt(id.trim(), 10));
    const label = match[2].trim();

    segments.push({
      type: "citation",
      content: match[0],
      sourceIds,
      label,
    });

    lastIndex = match.index + match[0].length;
  }

  // Add remaining text
  if (lastIndex < text.length) {
    segments.push({
      type: "text",
      content: text.slice(lastIndex),
    });
  }

  return segments;
}

/**
 * Render text with inline citations as React nodes
 */
export function renderWithCitations(
  text: string,
  sources: SourceInfo[]
): ReactNode[] {
  const segments = parseCitations(text);

  return segments.map((segment, idx) => {
    if (segment.type === "citation" && segment.sourceIds && segment.label) {
      return (
        <InlineCitation
          key={idx}
          sourceIds={segment.sourceIds}
          label={segment.label}
          sources={sources}
        />
      );
    }
    return <span key={idx}>{segment.content}</span>;
  });
}
```

---

### Step 7: Integrate Citations into Markdown Rendering

**File:** `frontend/src/components/research/research-document.tsx`

Update the markdown components to parse citations:

```tsx
import { renderWithCitations } from "@/lib/citation-parser";

// Create a custom text renderer that handles citations
const createMarkdownComponents = (sources: SourceInfo[]): Components => ({
  // Override paragraph to parse citations in text
  p: ({ children }) => {
    const processedChildren = processChildren(children, sources);
    return <p className="mb-3 last:mb-0">{processedChildren}</p>;
  },
  // Override other text-containing elements similarly
  li: ({ children }) => {
    const processedChildren = processChildren(children, sources);
    return <li className="pl-1">{processedChildren}</li>;
  },
  strong: ({ children }) => {
    const processedChildren = processChildren(children, sources);
    return <strong className="font-semibold">{processedChildren}</strong>;
  },
  // ... keep other component overrides
});

// Helper to process children and render citations
function processChildren(children: ReactNode, sources: SourceInfo[]): ReactNode {
  return React.Children.map(children, (child) => {
    if (typeof child === "string") {
      // Check if string contains citation markers
      if (child.includes("[cite:")) {
        return <>{renderWithCitations(child, sources)}</>;
      }
    }
    return child;
  });
}

// In the component, create components with sources context:
export function ResearchDocument({ report, ... }: Props) {
  const markdownComponents = useMemo(
    () => createMarkdownComponents(report.sources),
    [report.sources]
  );

  // Use markdownComponents in ReactMarkdown instances
  return (
    <ReactMarkdown
      remarkPlugins={[remarkGfm, remarkBreaks]}
      components={markdownComponents}
    >
      {section.content}
    </ReactMarkdown>
  );
}
```

---

### Step 8: Update Sources Display at Bottom

**File:** `frontend/src/components/research/research-document.tsx`

Update the sources card to show both cited and general sources:

```tsx
{/* Sources Card */}
<Card className="border-border/30">
  <CardHeader className="pb-3">
    <CardTitle className="text-base flex items-center gap-2">
      <FileText className="h-4 w-4" />
      Sources ({report.sources.length + (report.general_sources?.length || 0)})
    </CardTitle>
  </CardHeader>
  <CardContent className="space-y-4">
    {/* Cited Sources */}
    {report.sources.length > 0 && (
      <div>
        <h4 className="text-sm font-medium text-muted-foreground mb-2">
          Cited Sources
        </h4>
        <ul className="space-y-2">
          {report.sources.map((source) => (
            <li key={source.id} className="flex items-start gap-2">
              {source.favicon_url && (
                <img
                  src={source.favicon_url}
                  alt=""
                  className="w-4 h-4 mt-0.5 rounded-sm"
                />
              )}
              <div className="flex-1 min-w-0">
                <a
                  href={source.url}
                  target="_blank"
                  rel="noopener noreferrer"
                  className="text-sm text-primary hover:underline block truncate"
                >
                  {source.title || source.url}
                </a>
                <span className="text-xs text-muted-foreground">
                  [{source.id}] {source.site_name}
                </span>
              </div>
            </li>
          ))}
        </ul>
      </div>
    )}

    {/* General Sources */}
    {report.general_sources && report.general_sources.length > 0 && (
      <div>
        <h4 className="text-sm font-medium text-muted-foreground mb-2">
          Additional Sources
        </h4>
        <ul className="space-y-2 text-sm">
          {report.general_sources.map((url, i) => (
            <li key={i} className="flex items-start gap-2">
              <ExternalLink className="h-4 w-4 mt-0.5 flex-shrink-0 text-muted-foreground" />
              <a
                href={url}
                target="_blank"
                rel="noopener noreferrer"
                className="text-primary hover:underline break-all"
              >
                {url}
              </a>
            </li>
          ))}
        </ul>
      </div>
    )}
  </CardContent>
</Card>
```

---

### Step 9: Handle Follow-up Research Citations

**File:** `terminal-research/src/openai.rs` (or wherever follow-up logic lives)

When processing follow-up questions that update the document:

1. Pass existing sources with their IDs to the follow-up prompt
2. Allow the AI to add new sources (with new IDs starting after max existing ID)
3. Merge sources arrays, preserving IDs

```rust
// When doing follow-up research:
let max_existing_id = existing_sources.iter().map(|s| s.id).max().unwrap_or(0);

// New sources get IDs starting from max_existing_id + 1
let new_sources: Vec<SourceInfo> = new_urls
    .into_iter()
    .enumerate()
    .map(|(idx, url)| SourceInfo::new(max_existing_id + idx + 1, url))
    .collect();

// Combine for the updated report
let all_sources = [existing_sources, new_sources].concat();
```

**Update the follow-up prompt to include existing source context:**

```rust
let existing_sources_context = format!(
    "## Existing Sources (you may cite these)\n{}\n\n## New Sources (use IDs starting from {})\n{}",
    format_sources_for_prompt(&existing_sources),
    max_existing_id + 1,
    format_sources_for_prompt(&new_sources)
);
```

---

### Step 10: Testing Checklist

#### Backend Tests
- [ ] `SourceInfo::new()` extracts site_name from URL correctly
- [ ] `SourceInfo::new()` generates correct favicon URL
- [ ] Citation format is preserved through serialization/deserialization
- [ ] Metadata fetching handles timeouts and errors gracefully
- [ ] Follow-up research preserves existing source IDs

#### Frontend Tests
- [ ] `parseCitations()` correctly parses `[cite:1,2,3:Label]` format
- [ ] `parseCitations()` handles text without citations
- [ ] `parseCitations()` handles multiple citations in one string
- [ ] `InlineCitation` displays correct count (+N format)
- [ ] Hover popover shows all cited sources
- [ ] Click opens correct URL in new tab
- [ ] Citations render correctly in markdown content

#### Integration Tests
- [ ] New research generates citations in content
- [ ] Citations link to correct sources
- [ ] Follow-up research adds new citations without breaking existing ones
- [ ] Streaming content updates don't break citation rendering

---

### Step 11: Migration Considerations

Since we're not supporting backward compatibility:

1. **No migration needed** - old documents will show sources at bottom only
2. **Version check (optional)** - could add a `has_inline_citations: bool` field to detect new format
3. **Graceful fallback** - if `sources` is `string[]` instead of `SourceInfo[]`, render as before

---

## File Summary

| File | Changes |
|------|---------|
| `terminal-research/src/types.rs` | Add `SourceInfo` struct, update `SynthesizedReport` |
| `terminal-research/Cargo.toml` | Add `url`, `scraper` dependencies |
| `terminal-research/src/openai.rs` | Update prompts for citations, add metadata fetching |
| `frontend/src/lib/types.ts` | Add `SourceInfo` interface, update `SynthesizedReport` |
| `frontend/src/lib/citation-parser.tsx` | New file: citation parsing utilities |
| `frontend/src/components/research/inline-citation.tsx` | New file: citation badge component |
| `frontend/src/components/research/research-document.tsx` | Integrate citation rendering |

---

## Acceptance Criteria

1. ✅ Research documents show inline citation pills in the text
2. ✅ Pills display topic summary + source count (e.g., "Energy Drink Info +3")
3. ✅ Hovering shows popover with favicon, title, and site name for each source
4. ✅ Clicking a citation opens the first source in a new tab
5. ✅ Sources section at bottom shows both cited and general sources
6. ✅ Follow-up questions preserve existing citations and can add new ones
7. ✅ Citations work in all content areas (sections, executive summary if cited)
