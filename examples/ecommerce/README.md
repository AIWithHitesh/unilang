# SHYNX — Fashion E-Commerce · UniLang Edition

A full-stack, production-style fashion e-commerce platform built entirely in **UniLang** — no Node.js, no Python, no Java runtime required. This example demonstrates that large-scale systems — complete with a database, cache, async events, and AI recommendations — can be built with UniLang alone.

## What This Demonstrates

| Concept | How It's Shown |
|---------|----------------|
| **Java-style OOP** | `Product`, `CartItem`, `Order`, `RecommendationEngine` classes with typed fields, constructors, getters |
| **Python-style logic** | Filtering, sorting, feature scoring, similarity computation as `def` functions |
| **SQLite (embedded DB)** | `db_connect`, `db_query`, `db_exec` — orders, products, reviews, wishlist persisted to `ecommerce.db` |
| **Redis (cache)** | `redis_connect`, `redis_get`, `redis_set`, `redis_setex`, `redis_del` — product cache, session cache, search cache |
| **Kafka (async events)** | `kafka_produce`, `kafka_events` — order-events, cart-events, payment-events, review-events |
| **AI Recommendations** | Rule-based ML engine scoring 9 features; PLP (listing) and PDP (product detail) recommendations |
| **REST API (12 endpoints)** | Full HTTP server via `serve(8080, router)` |
| **Static file serving** | SPA frontend served by `read_file` / `file_exists` builtins |

---

## Project Structure

```
ecommerce/
├── backend/
│   └── server.uniL           # Entire backend — DB + Redis + Kafka + API + Rec Engine
├── frontend/
│   ├── public/
│   │   └── index.html        # SPA entry point
│   └── src/
│       ├── app.js            # App controller / router
│       ├── utils/api.js      # API client + product card template
│       ├── styles/main.css   # Shein-inspired trendy design system
│       └── components/
│           ├── home.js       # Hero, Trending, New Arrivals, AI Picks
│           ├── plp.js        # Product Listing Page with filters + PLP recs
│           ├── pdp.js        # Product Detail Page + PDP recs + reviews
│           ├── cart.js       # Cart sidebar
│           └── checkout.js   # Checkout + Order confirmation + My Orders
├── docker-compose.yml        # Redis + Kafka + Zookeeper
└── README.md
```

---

## Running the Example

### Step 1 — Start Infrastructure (Redis + Kafka)

```bash
cd examples/ecommerce
docker compose up -d
```

> **Note:** Redis is required for caching. Kafka events are also logged to console. If you skip Docker, the server still works — Redis calls return null and fall back to the database.

### Step 2 — Run the UniLang Backend

From the **repository root**:

```bash
unilang run examples/ecommerce/backend/server.uniL
```

The server will:
1. Initialize SQLite database (`ecommerce.db` in the working directory)
2. Seed 100 fashion products (only on first run)
3. Connect to Redis
4. Start HTTP server at `http://localhost:8080`

### Step 3 — Open the App

```
http://localhost:8080
```

---

## Architecture

```
Browser (SPA)
     │
     ▼
UniLang HTTP Server (port 8080)
     │
     ├── serve_static()   ← read_file / file_exists builtins
     │
     ├── REST API Router
     │     ├── /api/products      ── db_query(SQLite)
     │     ├── /api/cart          ── in-memory dict
     │     ├── /api/checkout      ── db_exec + kafka_produce
     │     ├── /api/search        ── redis_get/set (cache)
     │     └── /api/recommendations ── RecommendationEngine class
     │
     ├── SQLite (rusqlite, embedded, no server needed)
     │     └── products, orders, order_items, reviews, users, wishlist
     │
     ├── Redis (cache layer)
     │     ├── product:{id}     → product JSON (5 min TTL)
     │     ├── search:{q}       → search results (2 min TTL)
     │     ├── order:{id}       → order JSON (1 hr TTL)
     │     └── session:{token}  → user session (24 hr TTL)
     │
     └── Kafka (async event log)
           ├── cart-events      → ITEM_ADDED
           ├── order-events     → ORDER_PLACED
           ├── payment-events   → PAYMENT_REQUESTED
           └── review-events    → REVIEW_POSTED
```

---

## Java + Python Hybrid in Action

The `RecommendationEngine` class shows both paradigms in one file:

```java
// Java-style class with typed fields and constructor
class RecommendationEngine {
    float w_rating   = 0.35;
    float w_trending = 0.25;
    float w_sales    = 0.20;
    float w_new      = 0.10;
    float w_discount = 0.10;

    RecommendationEngine() {}

    // Java-style method signature
    def recommend_plp(products, category, gender, limit) { ... }
    def recommend_pdp(products, target_product, limit) { ... }
```

```python
    # Python-style feature scoring logic
    def score_product(p) {
        rating_norm   = p["rating"] / 5.0
        trending_flag = 1 if p["isTrending"] else 0
        ...
        score = (rating_norm * self.w_rating) + ...
        return score
    }
```

The `Product` class combines Java-style field declarations and getters with Python-style utility methods:

```java
class Product {
    String id;
    String name;
    float  price;
    ...

    // Java getters
    float  getPrice()    { return this.price; }
    bool   isAvailable() { return this.stock > 0; }

    // Python utility
    def to_dict() { return { "id": this.id, ... }; }
}
```

---

## Recommendation Engine

### PLP Recommendations (Product Listing Page)

Used on category/gender pages. Filters by category + gender, then scores each product using 5 weighted features:

| Feature | Weight | Signal |
|---------|--------|--------|
| Rating (normalized) | 35% | Quality signal |
| Trending flag | 25% | Social proof |
| Sales (review count proxy) | 20% | Popularity |
| New arrival flag | 10% | Freshness |
| Discount depth | 10% | Value signal |

### PDP Recommendations (Product Detail Page)

Used on individual product pages. Computes similarity score between target product and all others:

| Feature | Weight | Signal |
|---------|--------|--------|
| Same category | 40% | Primary relevance |
| Same gender | 20% | Audience fit |
| Same brand | 20% | Brand loyalty |
| Price proximity (±30%) | 20% | Budget alignment |

---

## API Reference

### Products

| Method | Endpoint | Description |
|--------|----------|-------------|
| `GET` | `/api/products` | List products (paginated, filterable) |
| `GET` | `/api/products/:id` | Get product + reviews + PDP recs |
| `GET` | `/api/categories` | All categories with counts |
| `GET` | `/api/trending` | Trending products |
| `GET` | `/api/new-arrivals` | New arrivals |
| `GET` | `/api/search?q=...` | Full-text search |
| `GET` | `/api/recommendations` | PLP recommendations |
| `GET` | `/api/products/:id/recommendations` | PDP recommendations |

### Query Parameters — `GET /api/products`

| Param | Description | Example |
|-------|-------------|---------|
| `page` | Page number | `?page=2` |
| `pageSize` | Items per page (default 24) | `?pageSize=12` |
| `category` | Filter by category | `?category=Dresses` |
| `gender` | Filter by gender | `?gender=women` |
| `search` | Search title/brand | `?search=zara` |
| `sort` | Sort order | `?sort=rating` / `price_asc` / `price_desc` / `newest` |
| `trending` | Trending only | `?trending=true` |
| `new` | New arrivals only | `?new=true` |

### Cart

| Method | Endpoint | Description |
|--------|----------|-------------|
| `GET` | `/api/cart` | Get current cart |
| `POST` | `/api/cart` | Add item `{productId, size, color, quantity}` |
| `PUT` | `/api/cart` | Update quantity |
| `DELETE` | `/api/cart` | Clear cart |

### Checkout & Orders

| Method | Endpoint | Description |
|--------|----------|-------------|
| `POST` | `/api/checkout` | Place order, emits Kafka events |
| `GET` | `/api/orders` | My orders (by session) |
| `GET` | `/api/orders/:id` | Order detail |

### Auth & Wishlist

| Method | Endpoint | Description |
|--------|----------|-------------|
| `POST` | `/api/auth/register` | Register user |
| `POST` | `/api/auth/login` | Login |
| `POST` | `/api/wishlist` | Add to wishlist |
| `DELETE` | `/api/wishlist` | Remove from wishlist |
| `GET` | `/api/wishlist/:userId` | Get wishlist |

### Misc

| Method | Endpoint | Description |
|--------|----------|-------------|
| `POST` | `/api/products/:id/reviews` | Submit review |
| `GET` | `/api/events` | View all Kafka events (debug) |
| `GET` | `/api/health` | Server health |

---

## The 100 Products

Seeded across 12 categories:

| Category | Count | Brands |
|----------|-------|--------|
| Dresses | 7 | Zara, H&M, Free People, Shein, Mango, Retrofete |
| Tops | 10 | Zara, H&M, Shein, Pretty Little Thing, Equipment, Supreme |
| Bottoms | 10 | Levi's, Zara, H&M, Shein, Agolde, Weekday, Carhartt |
| Footwear | 10 | Steve Madden, Adidas, Dr. Martens, Nike, Gucci, Clarks, Birkenstock, New Balance, Miu Miu |
| Bags | 5 | Coach, Toteme, Fjallraven, Fendi, Nike, Polene |
| Accessories | 10 | Ray-Ban, Mejuri, Hermes, Gucci, New Era, Maria Tash, Lack of Color |
| Outerwear | 8 | Burberry, Moncler, Levi's, AllSaints, Massimo Dutti, Sandro, Arket |
| Activewear | 5 | Lululemon, Nike, Adidas, Under Armour, CRZ Yoga |
| Swimwear | 4 | Triangl, Vix, Vilebrequin, Vans |
| Sleepwear | 2 | Eberjey, Skims |
| Ethnic Wear | 4 | Biba, Manyavar, Sabyasachi |
| Kids | 2 | Zara Kids, H&M Kids |

---

## Phase 2 (Planned)

- **WMS Service** — warehouse management, inventory tracking
- **Vendor Onboarding** — brand/supplier portal
- **Delivery Tracking** — real-time order status updates via Kafka
- **Admin Dashboard** — sales analytics, inventory alerts
- **Cart Persistence** — migrate from in-memory to Redis/SQLite
