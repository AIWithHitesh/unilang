# Library Management System — UniLang Edition

A full-stack library management application built entirely in **UniLang** — no Python, no Java, no Node.js. The HTTP server, REST API, business logic, prediction engine, and static file serving all run from a single UniLang script.

## Overview

This example demonstrates that a real, production-style web application can be built and run purely with UniLang. It includes:

- A **REST API** with 12 endpoints covering books, ratings, categories, languages, dashboard analytics, and ML-style predictions
- A **Single Page Application** frontend (vanilla JS) served directly by the UniLang server
- **50 books** in an in-memory store with full metadata: checkouts, ratings, late returns, computed prediction tags, and more
- A **prediction engine** (from `prediction_engine.uniL`) that tags every book at startup and re-tags on rating updates
- A **dashboard** with top checked-out books, top-rated books, category stats, late-return hotspots, and prediction tag distribution

## Project Structure

```
library-mgmt/
├── backend/
│   ├── api/
│   │   └── server.uniL              # Full HTTP server — the only file you need to run
│   ├── models/
│   │   └── book.uniL                # Book, Review, CheckoutTicket entity models
│   ├── ml/
│   │   └── prediction_engine.uniL   # PredictionEngine class (feature extraction + labelling)
│   └── services/
│       └── library_service.uniL     # Standalone library service demo
├── frontend/
│   ├── public/
│   │   └── index.html               # SPA entry point
│   └── src/
│       ├── app.js
│       ├── styles/main.css
│       ├── utils/api.js             # API client (calls localhost:8080)
│       └── components/
│           ├── dashboard.js
│           ├── books.js
│           ├── predictions.js
│           └── analytics.js
└── unilang.toml                     # Project configuration
```

## Running the Server

From the **repository root**:

```bash
unilang run examples/library-mgmt/backend/api/server.uniL
```

Then open your browser at:

```
http://localhost:8080
```

The frontend SPA loads automatically. The API is available at `http://localhost:8080/api`.

## Models

### `backend/models/book.uniL`

Defines three entity models in a hybrid Java/Python style:

#### `Book`
Full book entity with Java-style fields and Python-style utility methods:
- `get_late_return_rate()` — computes `lateReturns / (lateReturns + onTimeReturns)`
- `get_popularity_score()` — weighted composite: `rating×0.3 + checkouts×0.5 + reviews×0.2` (normalised)
- `checkout()` / `returnBook(isLate)` — mutate availability and return stats
- `addRating(rating)` — rolling average update
- Java-style getters / `setPredictionTag()`

#### `Review`
Stores user ratings with `bookId`, `userId`, `userName`, `rating`, `comment`, `reviewDate`.

#### `CheckoutTicket`
Tracks an active checkout with `checkoutDate`, `dueDate`, `returnDate`, and `isLate` flag.
- `markReturned(returnDate)` — sets `isLate = returnDate.isAfter(dueDate)`

Also defines enums: `BookCategory`, `BookStatus`, `PredictionTag`.

---

### `backend/ml/prediction_engine.uniL`

Defines the `PredictionEngine` class and two standalone functions used by `server.uniL`:

#### Feature Extraction (`extract_features`)
Computes per-book ML features:

| Feature | Description |
|---------|-------------|
| `totalCheckouts` | Raw checkout count |
| `lateReturnRate` | `lateReturns / totalReturns` |
| `averageRating` | Mean user rating |
| `totalRatings` | Number of ratings |
| `popularityScore` | Weighted composite (rating + checkouts + reviews) |
| `checkoutsPerCopy` | `totalCheckouts / totalCopies` |
| `ratingsPerCheckout` | Engagement ratio |
| `bookAge` | Years since publication |
| `isEnglish` | Binary language flag |

#### Label Generation (`generate_labels`)
Assigns a prediction tag to each book based on rule thresholds:

| Tag | Rule |
|-----|------|
| `most_likely_booked` | `checkouts ≥ 150` AND `rating ≥ 4.7` |
| `late_return` | `lateReturnRate > 0.09` AND `checkouts ≥ 60` |
| `on_time_return` | `lateReturnRate ≤ 0.05` AND `checkouts ≥ 60` |
| `less_likely_booked` | Everything else |

> **Note:** The original `PredictionEngine` class trains a Random Forest + Gradient Boosting ensemble (sklearn) to learn these rules. `server.uniL` implements the same labelling logic as a pure rule-based engine since UniLang's runtime does not yet support Python ML libraries.

#### `PredictionEngine` class (full ML version)
- `train(books)` — extracts features, encodes labels, trains RF + GBM, cross-validates
- `predict_books(books)` — ensemble prediction (RF + GBM probabilities averaged)
- `save_model(path)` / `load_model(path)` — joblib serialisation
- `get_model_info()` — returns accuracy, feature list, class distribution

---

### `backend/services/library_service.uniL`

A standalone UniLang script (run independently with `unilang run`) demonstrating pure service-layer logic: `find_book`, `search_books`, `get_stats`, `get_top_books`, `get_by_category`.

## API Endpoints

| Method | Endpoint | Description |
|--------|----------|-------------|
| `GET` | `/api/health` | Server health and book counts |
| `GET` | `/api/dashboard` | Full dashboard analytics |
| `GET` | `/api/books` | List books (paginated, filterable) |
| `GET` | `/api/books/:id` | Get a single book |
| `POST` | `/api/books` | Add a new book |
| `PUT` | `/api/books/:id/checkout` | Check out a book |
| `PUT` | `/api/books/:id/return` | Return a book |
| `POST` | `/api/books/:id/rate` | Submit a rating (1–5) |
| `GET` | `/api/categories` | List all categories |
| `GET` | `/api/languages` | List all languages |
| `GET` | `/api/predictions/stats` | Prediction model stats and tag distribution |
| `GET` | `/api/predictions/books` | Books with prediction tags and confidence scores |

### Query Parameters — `GET /api/books`

| Parameter | Description | Example |
|-----------|-------------|---------|
| `page` | Page number (default: 1) | `?page=2` |
| `pageSize` | Items per page (default: 20) | `?pageSize=10` |
| `category` | Filter by category | `?category=Fiction` |
| `language` | Filter by language | `?language=English` |
| `status` | Filter by status (`available` / `checked_out`) | `?status=available` |
| `search` | Search title or author | `?search=tolkien` |

### Query Parameters — `GET /api/predictions/books`

| Parameter | Description | Example |
|-----------|-------------|---------|
| `tag` | Filter by prediction tag | `?tag=most_likely_booked` |

Prediction tags: `most_likely_booked` · `late_return` · `on_time_return` · `less_likely_booked`

### `POST /api/books/:id/rate` — Request Body

```json
{ "rating": 4.5 }
```

Returns the updated `averageRating`, `totalRatings`, and recalculated `predictionTag` / `popularityScore`.

## Book Data Model

Each book carries both static fields (defined at startup) and **computed fields** set by the prediction engine:

**Static fields:**
```
id, title, author, isbn, category, language, status,
totalCopies, availableCopies, pageCount, publishYear, publisher,
totalCheckouts, lateReturns, onTimeReturns, averageRating, totalRatings
```

**Computed at startup by prediction engine:**
```
lateReturnRate      — Book.get_late_return_rate()
popularityScore     — Book.get_popularity_score()
predictionTag       — generate_labels() rule logic
predictionConfidence — confidence score for the assigned tag
bookAge             — 2026 - publishYear
checkoutsPerCopy    — totalCheckouts / totalCopies
ratingsPerCheckout  — totalRatings / totalCheckouts
```

## Dashboard Response Shape

```json
{
  "overview": {
    "totalBooks": 50,
    "availableBooks": 36,
    "checkedOutBooks": 14,
    "totalReviews": 2468,
    "averageRating": 4.51,
    "totalCheckouts": 5407
  },
  "topCheckedOut": [{ "title": "...", "author": "...", "checkouts": 240 }],
  "topRated":      [{ "title": "...", "author": "...", "rating": 4.9 }],
  "predictionTagDistribution": {
    "most_likely_booked": 6,
    "late_return": 11,
    "on_time_return": 5,
    "less_likely_booked": 28
  },
  "lateReturnHotspots": [{ "title": "...", "lateReturns": 22, "totalCheckouts": 240 }],
  "categoryStats": [{ "category": "Fiction", "available": 10, "checkedOut": 3, "total": 13 }]
}
```

## How It Works

The server is a single UniLang script (`server.uniL`) that:

1. Defines 50 books as an in-memory list of dicts (globals persist across requests)
2. Implements `compute_book_features()`, `predict_tag()`, and `apply_predictions()` — directly ported from `prediction_engine.uniL`'s feature extraction and label generation logic
3. Calls `apply_predictions()` at startup to set `predictionTag`, `predictionConfidence`, `lateReturnRate`, `popularityScore`, and other computed fields on all 50 books
4. Defines one handler function per endpoint, including `handle_rate_book()` which re-runs the prediction after each new rating (mirrors `Book.addRating()` from `book.uniL`)
5. Serves the frontend SPA's static files via `read_file()` and `file_exists()` builtins
6. Routes all requests through a `router(req)` function passed to `serve(8080, router)`

The built-in `serve()` function starts a TCP-based HTTP/1.1 server. Each request is passed to the router as a dict:

```
{ "method": "GET", "path": "/api/books", "query": "page=1&pageSize=20", "headers": {...}, "body": "" }
```

The router returns a response dict:

```
{ "status": 200, "body": "...", "content_type": "application/json" }
```

CORS headers (`Access-Control-Allow-Origin: *`) are automatically added by the runtime.
