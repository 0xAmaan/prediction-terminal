# Fey UI Kit Reference Guide

This document serves as a quick reference for implementing designs from the Fey UI Kit V1.0 (Unofficial - Community). It contains node IDs and structural information to help Claude Code access the correct Figma elements when building components.

## Figma File Info
- **File Key**: `lJZhaWTxa3MUANHQAitwEe`
- **Base URL**: `https://www.figma.com/design/lJZhaWTxa3MUANHQAitwEe/Fey-UI-kit-V1.0--unofficial----Community--Community-`

---

## 1. Foundations (Page: `401:104683`)

### Color System

#### Greys (Text, Strokes)
| Token | Hex | Use Case |
|-------|-----|----------|
| Grey-100 | `#EEF0F1` | Lightest grey |
| Grey-200 | `#D0D0D0` | Light grey |
| Grey-300 | `#B6BEC4` | Medium-light grey |
| Grey-400 | `#9AA4AD` | Medium grey |
| Grey-500 | `#7D8B96` | Neutral grey |
| Grey-600 | `#64727C` | Medium-dark grey |
| Grey-700 | `#4E5860` | Dark grey |
| Grey-800 | `#373E44` | Darker grey |
| Grey-900 | `#202427` | Darkest grey |

#### Background Colors
| Token | Hex | Use Case |
|-------|-----|----------|
| BG-100 | `#070709` | Primary dark background (deepest) |
| BG-200 | `#101116` | Secondary background |
| BG-300 | `#131419` | Card background |
| BG-400 | `#16181C` | Elevated background |
| BG-500 | `#1A1B20` | Surface background |
| BG-600 | `#131419` | Alternative surface |

#### Accent Colors
The accent palette mirrors the grey scale but with brand-specific tints. Additional accent colors include:
- Orange variants for highlights
- Gradient backgrounds

#### Gradients
- Available at node `401:104820` (Frame: "Gradient")
- Use cases: Background, Strokes

---

### Typography

**Primary Font**: Calibre (Node: `401:104837`)

#### Body Text (Node: `401:104840`)
Use cases: Body text, Paragraph, Tag

| Weight | Sizes |
|--------|-------|
| Regular | 10px, 12px, 14px, 16px, 18px |
| Medium | 10px, 12px, 14px, 16px, 18px |

#### Button Text (Node: `401:104861`)
Use case: Button text

| Weight | Size |
|--------|------|
| Regular | 14px |
| Regular UL (Underline) | 14px |
| Semibold | 14px |
| Semibold UL | 14px |

#### Tag Monospace (Node: `401:104883`)
Use cases: Tags, Highlight text

| Weight | Sizes |
|--------|-------|
| Regular | 10px, 12px, 14px |
| Medium | 10px, 12px, 14px |

#### Headings (Node: `401:104902`)
Use cases: Headers, Labels
Weight: Semibold

| Sizes |
|-------|
| 12px, 14px, 16px, 18px, 20px, 24px, 32px, 40px, 48px, 56px, 64px |

---

## 2. Elements (Page: `0:1`)

### Sidebar
- **Node ID**: `401:104921`
- **Logo Sizes**: 32px, 24px, 20px, 18px, 16px (Node: `394:1937`)
- **Icon Element States**: Default, Selected (Node: `263:17370`)
- **Full Sidebar Component**: Node `271:17503`

### Tags (Node: `401:104929`)

#### Tag Style Component (`_Tag style` - Node: `139:75`)
**Backgrounds**: White, Grey, Orange, Red, Purple, Yellow, Green, Berry, Blue
**Stroke Types**: None, Solid, Dash
**Padding**: Large, Small

#### Command Tags (Node: `290:33026`)
**Styles**: Black, Grey, Light Grey
Use cases: Keyboard shortcuts, code representation

#### Shortcut Tag (Node: `401:104939`)
States: Default, Hover, with Icon

#### Rounded Tag (Node: `86:12`)
States: Default, Hover
Use cases: Cards, Banners

#### Filter Tags (Node: `142:112`)
Positions: Start, Mid, End
Use cases: Table filter elements

### Tabs (Node: `401:104952`)

#### Filled Tabs (Node: `144:77`)
- `_Tab Block`: Default, Selected states
- Types: Single, Multiple

#### Regular Tabs (Node: `70:358`)
States: Default, Clicked, Hover
Positions: Bottom, Top
Sizes: Small, Medium

### Alert (Node: `401:104963`)
Styles: Default, Dark
- Default: For "coming soon" or general alerts
- Dark: For alerts requiring action

#### Alert Stack (Node: `93:1159`)
States: Collapsed, Expanded
Note: Hover triggers expansion in UI

### Checkbox (Node: `401:104976`)
States: Checked, Unchecked
Node: `284:911`

### Toggle (Node: `401:104982`)
States: Left (off), Right (on)
Node: `401:6492`

### Icons (Node: `447:305491`)
**Library**: Phosphor Icons 2.0 Regular
**Count**: 1,248 icons
**Source**: phosphoricons.com
**Node for icons grid**: `447:349600`

Common categories:
- Arrows & Navigation
- Communication (Chat, Phone, Mail)
- Media controls
- Brand logos (Apple, Amazon, Android, etc.)
- UI elements (Check, Warning, Info, etc.)
- Finance & Commerce

### Search Box (Node: `401:2557`)
Types:
1. Search Default (`401:2705`)
2. Search Input Large (`401:2775`)
3. Search Input Fit (`401:2809`)
4. Search Command (`401:2845`)
5. Search Command / Nothing Found (`401:2990`)

Components:
- `_Search block header` (Node: `277:956`)
- `_Search text` (Node: `276:927`)
- `_Email` (Node: `284:921`)

### Preferences Card (Node: `401:3292`)
States: Idle, Hover/Selected

**Media Types** (Node: `154:811`):
- None, Mail, Keyboard, Early adopt, Credit card

**Icon Types** (Node: `154:1029`):
- Feedback, Profile, Payment, Communication, Shortcuts

### Pricing Plans Card (Node: `401:4818`)
Types: Silver, Gold
Components:
- Price Separator
- Tag
- Price Big
- Price (Month/Year variants)

### News Cards (Node: `401:5427`)
Sizes:
- Full + tabs
- Medium
- Full + tags
- Medium + tags

### Table Row (Node: `401:5880`)
**Type 1** (Node: `176:33071`):
- States: Default, Hover, Loader with shimmer, Loader

**Type 2** (Node: `245:891`):
- States: Default, Hover

**Type 3** (Node: `160:654`):
- States: Default, Selected
- Edges: Rounded, None

### Charts (Node: `495:37313`)

#### Components:
- **Table Indication Tip**: Positive/Negative (Node: `495:37315`)
- **Bar**: Orange, White, Fade, White 180 (Node: `495:37324`)
- **Hover Slider**: Blur, Solid styles (Node: `495:37333`)
- **Progress Bar**: 3 variants (Node: `495:37365`)
- **Circles**: 2 variants (Node: `495:37519`)

#### Chart Types:
- Bar Graph (5 types)
- Line Chart (multiple types)

---

## 3. Illustrations (Node: `401:104927`)

Available illustrations (Node: `401:104928`):
- Envelope (`328:1160`)
- Early Adopter (`328:1121`)
- Keyboard Top (`328:1165`)
- Credit Cards:
  - VISA Card (`328:1061`)
  - Master Card (`328:1060`)
  - Discover (`328:1059`)
  - AMEX (`328:1058`)

---

## 4. How to Access Components

### Getting Design Context for a Specific Component
Use the `get_design_context` tool with:
```
fileKey: lJZhaWTxa3MUANHQAitwEe
nodeId: [node ID from this reference]
```

### Getting Screenshots
Use the `get_screenshot` tool with the same parameters. Note: Avoid requesting screenshots of very large nodes to prevent 400 errors.

### Example Queries:
1. **Get button styles**: Node `401:104861` (Button typography)
2. **Get color palette**: Node `401:104684` (Colors frame)
3. **Get tag variations**: Node `139:75` (_Tag style)
4. **Get table component**: Node `176:33071` (Type 1 table)
5. **Get chart components**: Node `495:37313` (Charts)

---

## 5. Design Principles (Inferred from UI Kit)

### Color Philosophy
- **Dark Theme First**: Primary backgrounds are very dark (#070709 to #1A1B20)
- **High Contrast**: Light text on dark backgrounds
- **Accent Sparingly**: Use colored accents for emphasis, not decoration

### Typography Rules
- **Calibre font family** throughout
- **Semibold for headings**, Regular/Medium for body
- **Monospace for tags** and technical content
- Size scale: 10, 12, 14, 16, 18, 20, 24, 32, 40, 48, 56, 64px

### Component Patterns
- **States**: Most components have Default, Hover, Selected/Active states
- **Rounded corners**: Subtle rounding on cards and buttons
- **Subtle shadows**: Used sparingly for elevation
- **Gradients**: Linear gradients for special backgrounds/strokes

### Layout
- **Card-based**: Information presented in contained cards
- **Dense but readable**: Efficient use of space
- **Consistent spacing**: Appears to use 8px grid system

---

## 6. Quick Reference for Common Tasks

| Task | Node ID | Tool to Use |
|------|---------|-------------|
| Get button styles | `401:104861` | get_design_context |
| Get input field | `276:927` | get_design_context |
| Get card component | `401:3292` | get_design_context |
| Get table row | `176:33071` | get_design_context |
| Get tag styles | `139:75` | get_design_context |
| Get chart | `495:37313` | get_design_context |
| Get icons | `447:349600` | get_metadata |
| Get sidebar | `271:17503` | get_design_context |
| Get full color system | `401:104684` | get_metadata |
| Get typography | `401:104837` | get_metadata |
| **Screens** | | |
| Get Preferences screen | `421:5552` | get_design_context |
| Get Payments modal | `423:2393` | get_design_context |
| Get Analysis screen | `435:12305` | get_design_context |
| Get Graphs screen | `427:5846` | get_design_context |
| Get Detail screen | `443:234092` | get_design_context |
| Get all screens overview | `340:1597` | get_metadata |

---

## 7. Tailwind CSS Color Mapping (Suggested)

```javascript
// tailwind.config.js colors suggestion
const feyColors = {
  grey: {
    100: '#EEF0F1',
    200: '#D0D0D0',
    300: '#B6BEC4',
    400: '#9AA4AD',
    500: '#7D8B96',
    600: '#64727C',
    700: '#4E5860',
    800: '#373E44',
    900: '#202427',
  },
  bg: {
    100: '#070709',
    200: '#101116',
    300: '#131419',
    400: '#16181C',
    500: '#1A1B20',
    600: '#131419',
  },
  // Add accent colors as discovered per component
}
```

---

## 8. Screens (Full Page Layouts)

**Screens Page Node ID**: `340:1597`

The Fey UI Kit contains complete screen designs for a finance/trading application.

### Available Screens

| Screen Name | Node ID | Dimensions | Description |
|-------------|---------|------------|-------------|
| **Preferences** | `421:5552` | 1440×775 | User account settings with preferences cards |
| **Preferences/Payments** | `423:2393` | 1440×775 | Payment method management with modal overlay |
| **Analysis** | `435:12305` | 1440×1044 | Economic analysis dashboard with charts |
| **Graphs** | `427:5846` | 1440×1435 | Stock comparison and graph selection tools |
| **Detail** | `443:234092` | 1440×4064 | Asset/stock detail page with full analytics |

---

### Screen Details

#### Preferences Screen (`421:5552`)
- Sidebar navigation (48px width)
- Header with title "Preferences" and user email
- Action buttons: "Keyboard shortcuts", "Log Out"
- Section: "Your account" with preference cards
- Components used:
  - `Sidebar` instance
  - `Preferences card` instances (×6)
  - `Button / Shadow / Text` instances
  - Pricing/subscription card on right side

#### Preferences/Payments Screen (`423:2393`)
- Same base as Preferences
- Modal overlay showing payment method
- Payment details: Card type (Visa), masked number, expiration
- "Update card" button
- Next payment date display

#### Analysis Screen (`435:12305`)
- Header: "Analysis" with date
- Tab navigation: Economics tabs with icons
- News cards section (507×285)
- Yield curve chart with data points
- "Economic calendar" section with placeholder state
- Components:
  - `News cards` instance
  - Line chart with Fed funds target range
  - Calendar placeholder with brand icons

#### Graphs Screen (`427:5846`)
- Header: "Graphs" with date and action buttons
- Graph selection panel (left side, 274px width):
  - "Compare graphs" section
  - Selected stocks: NVDA, AMZN, SPY, TYT
  - Comparison suggestions: VIX, AAPL, QQQ, GME, DIA, SHOP
- Main chart area (815×440):
  - Multi-line comparison chart
  - Date range: Sep 2023 - Sep 2024
  - Y-axis scale with values
- Time period tabs: D, W, M, 3M, 6M, Y, All
- Table section: "Your account" with stock data rows
- Components:
  - `_Type 2`, `_Type 3`, `_Type 4`, `_Type 5` table instances

#### Detail Screen (`443:234092`)
- Longest screen (4064px height)
- Back navigation with stock symbol and price
- Horizontal tabs: Info, Financials, Ratings, Options, Institutional, News, Earnings
- Action buttons: "Add to list", Share icon
- Stock info header: AMZN · Amazon Inc.
- Price display: $178.52 with change percentage
- Large line graph with gradient fill
- Volume activity bars below chart
- Extended content area for detailed information
- Dot pattern background decoration

---

### Screen Layout Pattern

All screens follow this structure:
```
┌─────────────────────────────────────────────┐
│ Sidebar │           Main Content            │
│  (48px) │                                   │
│         │  ┌─────────────────────────────┐  │
│  Logo   │  │         Header              │  │
│         │  │  Title + Subtitle + Actions │  │
│  Nav    │  ├─────────────────────────────┤  │
│  Icons  │  │         Content             │  │
│         │  │  Cards / Charts / Tables    │  │
│         │  │                             │  │
│         │  └─────────────────────────────┘  │
└─────────────────────────────────────────────┘
```

- **Sidebar**: Fixed 48px width, contains logo and navigation icons
- **Main content offset**: Starts at x=144 (48px sidebar + 96px padding)
- **Content width**: 1200px typical
- **Header height**: ~157px with title frame at y=56

---

### Accessing Screens

To get design context for any screen:
```
fileKey: lJZhaWTxa3MUANHQAitwEe
nodeId: [screen node ID from table above]
```

For specific sections within a screen, use the child node IDs listed in the screen details.

---

## 9. Additional Elements (Detailed)

### Buttons (Inferred from Components)

Based on the Search box and Preferences card patterns:

**Button with Shadow** - Commonly used for actions
- Appears in Search UI (`401:2735`, `401:2743`)
- Size: 20x20 (icon), 32x32 (with text)
- Has subtle shadow effect

**Shadow / Text Button** - Used in forms
- Node: `401:2823`
- Combines icon + text

### Input Fields

**Search Text Input** (`276:927`)
States by Property:
- Default (Large/Small)
- Click (Large/Small)
- Typing (Large/Small)
- Typed (Large/Small)

Large size: 196x65
Small size: varies (198-226 width, 68 height)

### Logo Component (`394:1937`)

Fey logo in multiple sizes:
| Size | Node ID |
|------|---------|
| 32px | `394:1938` |
| 24px | `394:1941` |
| 20px | `394:1944` |
| 18px | `2003:3004` |
| 16px | `394:1947` |

### Payment Method UI (`401:4636`)

Shows credit card display pattern:
- Card type (Visa, etc.)
- Masked number (dots + last 4)
- Expiration date
- "Update card" button
- Next payment info

---

## 10. Component Hierarchy

Understanding how components nest:

```
Screen
├── Sidebar (401:104921)
│   ├── Logo (394:1937)
│   └── Icon Elements (263:17370)
├── Main Content
│   ├── Search Box (401:2557)
│   │   ├── Search Header (277:956)
│   │   └── Search Text (276:927)
│   ├── Cards
│   │   ├── Preferences Card (401:3292)
│   │   ├── Pricing Card (401:4818)
│   │   └── News Card (401:5427)
│   ├── Tables (401:5880)
│   │   └── Table Rows (176:33071, 245:891, 160:654)
│   └── Charts (495:37313)
│       ├── Bar Charts
│       └── Line Charts
└── Alerts (401:104963)
    └── Alert Stack (93:1159)
```

---

## Notes for Claude Code

1. **Always use `get_design_context`** when implementing a specific component - it provides CSS/code suggestions
2. **Use `get_metadata`** when you need to explore structure or find child node IDs
3. **Avoid `get_screenshot`** for large nodes (causes 400 errors with images > 8000px)
4. **Font**: The UI kit uses "Calibre" - consider using Inter or similar if Calibre isn't available
5. **Icons**: Reference Phosphor Icons (phosphoricons.com) for icon implementation
6. **When user provides a Figma URL**: Extract the node-id and use it directly - don't guess
7. **For screens**: Break down into component parts and implement section by section
