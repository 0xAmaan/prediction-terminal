# Arbitrage Alerts

## Overview

Once markets are matched across platforms (see `CROSS_PLATFORM_MATCHING.md`), we can monitor price differences and alert when arbitrage opportunities arise. This document outlines the alerting system.

## Prerequisites

- Cross-platform matching must be implemented (Phase 1 minimum)
- Markets must be linked before spreads can be calculated

## What is Prediction Market Arbitrage?

When the same event is priced differently across platforms:

```
Kalshi:     "Bitcoin > $100K by Dec 31" → YES: 45¢, NO: 55¢
Polymarket: "Bitcoin > $100K by Dec 31" → YES: 48¢, NO: 52¢

Opportunity: Buy YES on Kalshi (45¢), Sell YES on Polymarket (48¢)
Profit: 3¢ per share (minus fees, slippage, capital lockup)
```

Real-world considerations:
- **Fees**: Both platforms charge trading fees (typically 1-2%)
- **Slippage**: Large orders move price
- **Capital lockup**: Money tied up until resolution
- **Execution risk**: Prices may move before both trades execute
- **Resolution risk**: Platforms may resolve differently

## Current State

### Existing code
- `UnifiedMarket` has `calculate_spread()` method (`terminal-core/src/market.rs`)
- Returns absolute difference in YES prices
- Not currently called anywhere

### What's missing
- Spread monitoring loop
- Threshold configuration
- Alert delivery system
- Historical tracking
- Dedicated UI

## Proposed Solution

### Component Architecture

```
┌─────────────────┐     ┌──────────────────┐     ┌─────────────────┐
│ Market Matcher  │────▶│  Spread Monitor  │────▶│  Alert Manager  │
│ (unified pairs) │     │  (calculates)    │     │  (notifies)     │
└─────────────────┘     └──────────────────┘     └─────────────────┘
                               │                        │
                               ▼                        ▼
                        ┌──────────────────┐     ┌─────────────────┐
                        │  Spread History  │     │  Notifications  │
                        │  (SQLite)        │     │  (UI, webhook)  │
                        └──────────────────┘     └─────────────────┘
```

### Phase 1: Spread Calculation & Display

Add spread calculation to unified market view:

```rust
// terminal-services/src/spread_monitor.rs
pub struct SpreadMonitor {
    market_service: Arc<MarketService>,
    matcher: Arc<MarketMatcher>,
}

impl SpreadMonitor {
    /// Get all matched markets with their current spreads
    pub async fn get_spreads(&self) -> Result<Vec<SpreadInfo>, TerminalError> {
        let unified = self.market_service.get_unified_markets().await?;

        unified
            .iter()
            .filter(|m| m.is_matched())
            .map(|m| SpreadInfo {
                mapping_id: m.mapping_id.clone(),
                kalshi_yes_price: m.kalshi_market.as_ref().map(|k| k.yes_price),
                polymarket_yes_price: m.polymarket_market.as_ref().map(|p| p.yes_price),
                spread: m.calculate_spread(),
                spread_percent: m.calculate_spread_percent(),
                best_buy: m.best_yes_price(),
                best_sell: m.best_no_price(),
                updated_at: Utc::now(),
            })
            .collect()
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct SpreadInfo {
    pub mapping_id: String,
    pub kalshi_yes_price: Option<Decimal>,
    pub polymarket_yes_price: Option<Decimal>,
    pub spread: Option<Decimal>,
    pub spread_percent: Option<Decimal>,
    pub best_buy: Option<(Platform, Decimal)>,
    pub best_sell: Option<(Platform, Decimal)>,
    pub updated_at: DateTime<Utc>,
}
```

**API endpoint**:
```
GET /api/arbitrage/spreads
```

**Frontend**: Dedicated arbitrage page showing all matched markets with spreads, sortable by spread size.

### Phase 2: Threshold Alerts

Add configurable alert thresholds:

```rust
// terminal-services/src/alert_manager.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertConfig {
    /// Minimum spread percentage to trigger alert
    pub min_spread_percent: Decimal,
    /// Minimum absolute spread in cents
    pub min_spread_cents: Decimal,
    /// Cooldown between alerts for same market (seconds)
    pub cooldown_seconds: u64,
    /// Whether alerts are enabled
    pub enabled: bool,
}

impl Default for AlertConfig {
    fn default() -> Self {
        Self {
            min_spread_percent: Decimal::new(3, 0),  // 3%
            min_spread_cents: Decimal::new(2, 2),    // $0.02
            cooldown_seconds: 300,                    // 5 minutes
            enabled: true,
        }
    }
}

pub struct AlertManager {
    config: AlertConfig,
    last_alerts: HashMap<String, DateTime<Utc>>,
    tx: broadcast::Sender<Alert>,
}

impl AlertManager {
    pub fn check_and_alert(&mut self, spread: &SpreadInfo) -> Option<Alert> {
        if !self.config.enabled {
            return None;
        }

        let spread_val = spread.spread?;
        let spread_pct = spread.spread_percent?;

        // Check thresholds
        if spread_pct < self.config.min_spread_percent
            && spread_val < self.config.min_spread_cents
        {
            return None;
        }

        // Check cooldown
        if let Some(last) = self.last_alerts.get(&spread.mapping_id) {
            if Utc::now() - *last < Duration::seconds(self.config.cooldown_seconds as i64) {
                return None;
            }
        }

        // Create alert
        let alert = Alert {
            id: Uuid::new_v4().to_string(),
            mapping_id: spread.mapping_id.clone(),
            spread: spread_val,
            spread_percent: spread_pct,
            best_buy: spread.best_buy.clone(),
            best_sell: spread.best_sell.clone(),
            timestamp: Utc::now(),
        };

        self.last_alerts.insert(spread.mapping_id.clone(), Utc::now());
        let _ = self.tx.send(alert.clone());

        Some(alert)
    }
}
```

### Phase 3: Notification Channels

Support multiple notification methods:

#### 1. In-App Notifications (WebSocket)

Push alerts to connected frontend clients:

```rust
// In WebSocket handler
pub enum OutgoingMessage {
    // ... existing variants
    ArbitrageAlert(Alert),
}
```

Frontend shows toast/banner when alert received.

#### 2. Webhook (Discord, Slack, etc.)

```rust
pub struct WebhookNotifier {
    url: String,
    client: reqwest::Client,
}

impl WebhookNotifier {
    pub async fn send(&self, alert: &Alert) -> Result<(), Error> {
        let payload = json!({
            "content": format!(
                "🚨 **Arbitrage Alert**\n{}\nSpread: {:.1}% (${:.2})\nBuy {} @ {}\nSell {} @ {}",
                alert.mapping_id,
                alert.spread_percent,
                alert.spread,
                alert.best_buy.0, alert.best_buy.1,
                alert.best_sell.0, alert.best_sell.1,
            )
        });

        self.client.post(&self.url)
            .json(&payload)
            .send()
            .await?;

        Ok(())
    }
}
```

#### 3. Email (Optional)

Lower priority - webhook covers most use cases.

### Phase 4: Historical Tracking

Store spread history for analysis:

```sql
CREATE TABLE spread_history (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  mapping_id TEXT NOT NULL,
  kalshi_yes_price REAL,
  polymarket_yes_price REAL,
  spread REAL,
  spread_percent REAL,
  recorded_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
  FOREIGN KEY (mapping_id) REFERENCES market_mappings(id)
);

CREATE INDEX idx_spread_mapping ON spread_history(mapping_id, recorded_at);
```

Background job records spreads every N minutes for matched markets.

**Use cases**:
- Chart spread over time
- Identify patterns (e.g., spreads widen at market close)
- Backtest alert thresholds

## Frontend Components

### Arbitrage Dashboard (`/arbitrage`)

```tsx
// frontend/src/app/arbitrage/page.tsx
export default function ArbitragePage() {
  const { data: spreads } = useQuery({
    queryKey: ["arbitrage", "spreads"],
    queryFn: fetchSpreads,
    refetchInterval: 10_000, // Refresh every 10s
  });

  return (
    <div>
      <h1>Arbitrage Opportunities</h1>

      {/* Alert config */}
      <AlertConfigPanel />

      {/* Current spreads table */}
      <SpreadsTable spreads={spreads} />

      {/* Recent alerts */}
      <RecentAlerts />
    </div>
  );
}
```

### Spreads Table

| Market | Kalshi | Polymarket | Spread | Action |
|--------|--------|------------|--------|--------|
| BTC $100K | 45¢ | 48¢ | 3¢ (6.3%) | View |
| S&P 6000 | 52¢ | 51¢ | 1¢ (1.9%) | View |

Features:
- Sort by spread (absolute or percent)
- Filter by minimum spread
- Click to view market details
- Highlight when above threshold

### Alert Toast

When WebSocket receives arbitrage alert:

```tsx
// In WebSocket provider
useEffect(() => {
  if (message.type === "arbitrage_alert") {
    toast({
      title: "Arbitrage Opportunity",
      description: `${message.mapping_id}: ${message.spread_percent}% spread`,
      action: <Button onClick={() => navigate(`/market/${message.mapping_id}`)}>View</Button>,
    });
  }
}, [message]);
```

## Configuration

Environment variables / settings:

```toml
# config.toml
[arbitrage]
enabled = true
min_spread_percent = 3.0
min_spread_cents = 0.02
cooldown_seconds = 300
check_interval_seconds = 30

[arbitrage.webhook]
enabled = true
url = "https://discord.com/api/webhooks/..."

[arbitrage.history]
enabled = true
record_interval_seconds = 60
retention_days = 30
```

## Data Flow Example

1. **Spread Monitor** runs every 30 seconds
2. Fetches prices for all matched markets
3. Calculates spread for each pair
4. **Alert Manager** checks against thresholds
5. If threshold exceeded and not in cooldown:
   - Creates alert record
   - Broadcasts to WebSocket clients
   - Sends webhook notification
6. **Spread History** records current spreads for analysis
7. Frontend updates spreads table in real-time

## Files to Create/Modify

| File | Action |
|------|--------|
| `terminal-services/src/spread_monitor.rs` | **NEW** - Spread calculation |
| `terminal-services/src/alert_manager.rs` | **NEW** - Alert logic |
| `terminal-services/src/webhook_notifier.rs` | **NEW** - Webhook delivery |
| `terminal-api/src/routes/arbitrage.rs` | **NEW** - API endpoints |
| `frontend/src/app/arbitrage/page.tsx` | **NEW** - Dashboard |
| `frontend/src/components/arbitrage/spreads-table.tsx` | **NEW** |
| `frontend/src/components/arbitrage/alert-config.tsx` | **NEW** |

## Success Criteria

1. Spreads calculated and displayed for all matched markets
2. Alerts triggered when spread exceeds threshold
3. In-app notifications appear in real-time
4. Webhook notifications delivered to external services
5. Historical spread data stored for analysis
6. Alert cooldown prevents notification spam
