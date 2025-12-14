# News Sources Analysis & Improvement Plan

## Current News Infrastructure

### Overview
The prediction terminal aggregates news from multiple sources with a focus on **real-time relevance** to active prediction markets. The system uses a **multi-tier approach** with caching and intelligent filtering.

### Current Sources

#### 1. RSS Feeds (Primary for Global News)
**Purpose**: Baseline news coverage with source diversity

**Wire Services** (Most reliable for breaking news):
- AP News - `https://feedx.net/rss/ap.xml`
- Reuters - Via RSSHub proxy

**Financial News**:
- CNBC Top News - Combined CMS feed
- Bloomberg - Via Exa.ai whitelist only

**Political News**:
- Politico - `https://www.politico.com/rss/politicopicks.xml`
- The Hill - `https://thehill.com/feed/`
- NPR Politics - `https://feeds.npr.org/1014/rss.xml`
- Axios - `https://api.axios.com/feed/`
- RealClearPolitics - Aggregates polls and betting odds

**General News**:
- BBC News - `https://feeds.bbci.co.uk/news/rss.xml`
- BBC World - `https://feeds.bbci.co.uk/news/world/rss.xml`
- NPR News - `https://feeds.npr.org/1001/rss.xml`
- Guardian US - `https://www.theguardian.com/us-news/rss`
- CBS News - `https://www.cbsnews.com/latest/rss/main`
- ABC News - `https://abcnews.go.com/abcnews/topstories`

**Tech News** (AI-focused):
- MIT Technology Review - `https://www.technologyreview.com/feed/`

**Crypto**:
- CoinDesk - `https://www.coindesk.com/arc/outboundfeeds/rss/`
- Decrypt - `https://decrypt.co/feed`
- CryptoPanic - Optional with API key, aggregates tweets/news

**Sports** (for betting markets):
- ESPN - General, NFL, NBA feeds
- CBS Sports - `https://www.cbssports.com/rss/headlines/`

**Space** (for SpaceX/NASA markets):
- NASA - `https://www.nasa.gov/rss/dyn/breaking_news.rss`

**Economics & Regulation**:
- Federal Reserve - `https://www.federalreserve.gov/feeds/press_all.xml`
- SEC Press Releases - `https://www.sec.gov/news/pressreleases.rss`

**International**:
- AP World News - Via RSSHub
- Al Jazeera - `https://www.aljazeera.com/xml/rss/all.xml`
- Guardian World - `https://www.theguardian.com/world/rss`
- DW News - `https://rss.dw.com/rdf/rss-en-all`

**Caching**: 60 seconds (RSS feeds don't update faster)
**Deduplication**: Normalized title matching
**Source Diversity**: Round-robin selection to prevent source clustering

#### 2. Google News RSS (Primary for Market-Specific News)
**Purpose**: Dynamic, market-targeted news with excellent freshness

**Implementation**:
- Uses `https://news.google.com/rss/search` with custom queries
- Builds optimized queries from market titles and outcomes
- Includes both regular news AND high-engagement Twitter/X posts
- Extracts key entities (people, companies, events) for precise matching

**Advantages**:
- ✅ **Free and unlimited** (no API key required)
- ✅ **Very current** (updates within minutes)
- ✅ **Excellent search relevance** (Google's semantic matching)
- ✅ **Broad coverage** (aggregates from thousands of sources)
- ✅ **Twitter integration** via site search (captures market-moving tweets)

**Query Building Strategy**:
1. Extract proper nouns from market title
2. Add context terms from outcomes (skip price targets)
3. Add domain-specific keywords (e.g., "emissions" for climate markets)
4. Expand abbreviations (US → United States)
5. Limit to 10 focused terms

**Caching**: 5 minutes for good results, 1 minute for empty results

#### 3. Exa.ai (Optional Semantic Search)
**Purpose**: Fallback for markets where Google News returns insufficient results

**Implementation**:
- Neural search with domain whitelist
- Only credible sources (NYT, Reuters, Bloomberg, BBC, etc.)
- Extracts dates from URLs (Exa's metadata is often wrong)

**Advantages**:
- ✅ High-quality sources only
- ✅ Good semantic understanding
- ✅ Find Similar API for related articles

**Limitations**:
- ❌ Requires API key (paid)
- ❌ Limited daily quota
- ❌ Sometimes returns old articles despite date filters

**Domain Whitelist** (48 sources):
- Top-tier: NYT, WSJ, WaPo, FT, Economist
- Wire: Reuters, AP, Bloomberg, AFP
- Broadcast: BBC, CNN, NBC, CBS, ABC, NPR
- International: Al Jazeera, DW, France24, Euronews
- Quality online: Guardian, Politico, Axios, The Hill
- Tech: TechCrunch, The Verge, Wired, Ars Technica
- Sports: ESPN, The Athletic
- Science: Scientific American, Nature

#### 4. Discord Integration (Optional)
**Purpose**: Track community sentiment and market-moving discussions

**Status**: Implemented but requires Discord API credentials

## Current Filtering & Matching

### Relevance Scoring
**Minimum relevance score**: 0.35 (raised from 0.15)

**Market-Specific Filtering**:
1. **Geographic validation**: US markets require "United States/US/America" mentions
2. **Multi-term matching**: Requires 1-2 key terms depending on market type
3. **Title length validation**: Minimum 15 characters (filters parsing errors)
4. **Age filtering**: Adaptive (30/90/180 days based on result count)
5. **Diversity enforcement**: Limits articles per outcome (2-4 depending on market size)

### Global News Filtering
**STRICT entity matching** - only shows news mentioning trending market topics

**Process**:
1. Get top 30 trending markets by volume
2. Extract entities (Trump, Bitcoin, Ukraine, etc.)
3. Filter RSS items to only those with entities in title
4. Generate dynamic Google News feeds for top 5 trending markets
5. Bonus scoring for entity prominence and recency

## Performance Characteristics

### Update Frequencies
- RSS feeds: **60s cache** (feeds update every 1-5 minutes)
- Google News market search: **5 min cache for good results, 1 min for empty**
- Global news feed: **5s cache** (very responsive)
- Market list: **2 min cache**

### Typical Response Times
- RSS fetch: ~2-3 seconds (parallel fetching of 25+ feeds)
- Google News search: ~1-2 seconds
- Exa.ai search: ~2-4 seconds
- Full market news request: ~3-5 seconds (first request, then cached)

## Issues & Limitations

### Current Problems

1. **No Semantic Understanding**
   - Relies on exact keyword matching
   - Misses related news with different terminology
   - Example: "Federal Reserve" vs "Fed" vs "FOMC" vs "Jerome Powell"

2. **No Cross-Market Intelligence**
   - News about "Bitcoin regulation" is relevant to both BTC price AND crypto politics markets
   - Currently requires market-specific searches, duplicating effort

3. **Limited Source Discovery**
   - Fixed list of RSS feeds
   - Can't adapt to new high-quality sources emerging
   - Crypto/sports markets may need niche specialized sources

4. **No Temporal Awareness**
   - Treats all news equally regardless of market resolution date
   - Markets closing in 24h should prioritize very recent news

5. **Redundancy in Sports Markets**
   - Sports markets often show generic league news rather than player-specific news
   - "NFL MVP" markets get "NFL standings" articles

6. **Twitter Coverage Gaps**
   - Google News only indexes high-engagement tweets (10k+ impressions)
   - Misses early signals from credible small accounts
   - No direct Twitter API integration

## Improvement Recommendations

### High Priority (Implement First)

#### 1. **Add Perplexity API** (Most Impactful)
**Why**: Perplexity is built specifically for real-time news aggregation and has better freshness than all current sources.

**Implementation**:
- Use Perplexity's `/search` endpoint with `news` focus mode
- Query construction similar to Google News (entity extraction)
- Set recency parameter to "24h" or "7d" based on market urgency
- Fallback to Google News if Perplexity quota exceeded

**Advantages**:
- ✅ Better than Google News for very recent news (last 1-6 hours)
- ✅ Includes real-time Twitter/X analysis
- ✅ Cites sources with timestamps
- ✅ Can ask follow-up questions for context

**Cost**: ~$5-20/month for typical usage

#### 2. **Add NewsAPI.org**
**Why**: Massive source coverage (150k+ sources) with good categorization

**Implementation**:
- Use `/everything` endpoint for market-specific searches
- Use `/top-headlines` with category filters for global feed
- Combine with existing sources for broader coverage

**Advantages**:
- ✅ 150,000+ sources globally
- ✅ Very fresh (updates every 15 minutes)
- ✅ Good filtering (language, domain, date range)
- ✅ Free tier: 100 requests/day

**Free Tier Limit**: 100 requests/day (need caching)

#### 3. **Add Twitter/X API v2** (Essential Stream)
**Why**: Direct access to market-moving tweets before Google indexes them

**Implementation**:
- Use Twitter Search API with market entities as queries
- Filter by engagement threshold (1k+ likes/retweets)
- Filter by verified accounts or known influencers
- Combine with Google News Twitter results

**Advantages**:
- ✅ Earliest signal (often 30-60 min before news sites)
- ✅ Direct from source (no intermediary)
- ✅ Can track specific influential accounts

**Cost**: Twitter API Basic ($100/month) or Free tier (very limited)

#### 4. **Better Crypto Sources**
Current crypto coverage is weak. Add:
- **CoinTelegraph RSS**: `https://cointelegraph.com/rss`
- **The Block**: `https://www.theblock.co/rss.xml`
- **CryptoSlate**: `https://cryptoslate.com/feed/`
- **Bitcoin Magazine**: `https://bitcoinmagazine.com/feed`
- **Messari**: Via API (requires key)

#### 5. **Sports-Specific Sources**
Add dedicated feeds:
- **NFL**: `https://www.nfl.com/feeds/rss/news` (official)
- **NBA**: `https://www.nba.com/rss/nba_rss.xml` (official)
- **MLB**: `https://www.mlb.com/feeds/news/rss.xml` (official)
- **Bleacher Report**: `https://bleacherreport.com/articles/feed`
- **Yahoo Sports**: Via RSS

### Medium Priority

#### 6. **Reddit Integration**
Track high-quality prediction market subreddits:
- r/bitcoin, r/CryptoCurrency (for crypto markets)
- r/nfl, r/nba (for sports markets)
- r/politics, r/PoliticalDiscussion (for election markets)
- r/wallstreetbets (for finance/stock markets)

**Implementation**: Use Reddit API or Pushshift dumps

#### 7. **Financial Data Feeds**
For markets about stocks, economy:
- **Yahoo Finance**: Market data + news RSS
- **MarketWatch**: `https://www.marketwatch.com/rss/`
- **Seeking Alpha**: Via RSS or API
- **Financial Times**: Already in Exa whitelist, add RSS

#### 8. **Prediction Market Competitor Monitoring**
Monitor what other prediction markets are highlighting:
- Track Polymarket's "trending" section
- Monitor Kalshi's featured markets
- Manifold Markets trending

### Low Priority (Nice to Have)

#### 9. **Regional/Language-Specific Sources**
For international markets:
- **Spanish**: EL PAÍS, El Mundo
- **French**: Le Monde, Le Figaro
- **German**: Der Spiegel, Die Zeit
- **Chinese**: South China Morning Post (English)
- **Japanese**: Japan Times (English)
- **Arabic**: Al Arabiya (English)

#### 10. **Academic/Scientific Sources**
For science/climate markets:
- **arXiv**: Preprints (physics, CS, math)
- **PubMed**: Medical research
- **Science Daily**: `https://www.sciencedaily.com/rss/all.xml`
- **Phys.org**: `https://phys.org/rss-feed/`

## Recommended Implementation Priority

### Phase 1: Better Real-Time Coverage (Week 1)
1. ✅ Add Perplexity API (2-3 hours implementation)
2. ✅ Add NewsAPI.org (1-2 hours implementation)
3. ✅ Add crypto-specific RSS feeds (30 min)
4. ✅ Add sports official feeds (30 min)

**Expected Impact**: 40-60% improvement in news freshness and coverage

### Phase 2: Social Signal Integration (Week 2)
1. ✅ Add Twitter API v2 integration (3-4 hours)
2. ✅ Add Reddit monitoring (2-3 hours)
3. ✅ Improve Discord integration documentation

**Expected Impact**: Catch news 30-120 minutes earlier than RSS

### Phase 3: Semantic Understanding (Week 3-4)
**This is the EMBEDDING + SEMANTIC MAPPING phase**
1. ✅ Generate embeddings for all markets
2. ✅ Embed incoming news articles
3. ✅ Semantic similarity matching
4. ✅ Cross-market news discovery

See detailed plan in next section.

---

## Next Steps: Embedding + Semantic Mapping

The current keyword-based approach will be augmented (not replaced) with **vector embeddings** for semantic understanding. This enables:

1. **Semantic News Matching**: Match news to markets by meaning, not just keywords
2. **Cross-Market Intelligence**: Discover news relevant to multiple markets
3. **Entity Resolution**: Understand "Fed" = "Federal Reserve" = "Jerome Powell"
4. **Temporal Awareness**: Weight recency and urgency intelligently

**Implementation details**: See `docs/embedding-semantic-mapping-plan.md`
