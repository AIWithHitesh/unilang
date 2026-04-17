# UniLang Cookbook

Common patterns for building production-ready UniLang HTTP services. Each recipe is self-contained and runnable with `unilang run <file>.uniL`.

---

## Table of Contents

1. [Authentication — JWT-style token auth](#recipe-1-authentication--jwt-style-token-auth)
2. [Pagination — paginated list and DB results](#recipe-2-pagination--paginated-list-and-db-results)
3. [Caching with Redis — cache-aside pattern](#recipe-3-caching-with-redis--cache-aside-pattern)
4. [Rate Limiting — per-IP sliding window](#recipe-4-rate-limiting--per-ip-sliding-window)
5. [Background Jobs with Kafka — producer/consumer](#recipe-5-background-jobs-with-kafka--producerconsumer)
6. [Input Validation — required fields, types, lengths](#recipe-6-input-validation--required-fields-types-lengths)
7. [Error Handling Middleware — consistent JSON errors](#recipe-7-error-handling-middleware--consistent-json-errors)
8. [Environment-Based Config — env vars with defaults](#recipe-8-environment-based-config--env-vars-with-defaults)

---

## Recipe 1: Authentication — JWT-style token auth

A lightweight token-auth pattern that avoids external JWT libraries. A token is the SHA-256-like hash of `username + secret + timestamp`; it is stored in Redis with a TTL so tokens expire automatically. A `require_auth` helper validates the `Authorization: Bearer <token>` header and can be called at the top of any protected handler.

**When to use:** Any endpoint that must be limited to logged-in users. Because token verification is a single Redis lookup it is fast and stateless from the application's point of view.

```unilang
// auth_demo.uniL — JWT-style token auth pattern
// Run: unilang run auth_demo.uniL

// ── Config ────────────────────────────────────────────────────

APP_SECRET = "change-me-in-production";
TOKEN_TTL  = 3600;   // 1 hour

// ── Helpers ───────────────────────────────────────────────────

// Simple deterministic "hash": combine fields with secret and
// take a substring of the result.  Replace with a real HMAC
// builtin once available in your UniLang build.
def make_token(username, timestamp) {
    raw = username + ":" + APP_SECRET + ":" + str(timestamp);
    // hash_sha256 is a UniLang builtin string utility
    return hash_sha256(raw);
}

def json_ok(data) {
    return {"status": 200, "body": to_json(data), "content_type": "application/json"};
}

def json_err(code, msg) {
    return {"status": code, "body": to_json({"error": msg, "code": code}), "content_type": "application/json"};
}

// Extract the Bearer token from the Authorization header.
def extract_bearer(req) {
    auth_header = req["headers"]["authorization"];
    if auth_header == null { return ""; }
    if auth_header == ""   { return ""; }
    parts = split(auth_header, " ");
    if len(parts) < 2 { return ""; }
    if lower(parts[0]) != "bearer" { return ""; }
    return parts[1];
}

// Middleware helper: returns the session dict on success,
// or null when the token is missing / expired / unknown.
def require_auth(req) {
    token = extract_bearer(req);
    if token == "" { return null; }
    session_json = redis_get("session:" + token);
    if session_json == null { return null; }
    return from_json(session_json);
}

// ── Handlers ──────────────────────────────────────────────────

def handle_register(req) {
    body     = from_json(req["body"]);
    username = body["username"];
    password = body["password"];

    if username == null { return json_err(400, "username required"); }
    if password == null { return json_err(400, "password required"); }
    if username == ""   { return json_err(400, "username required"); }
    if password == ""   { return json_err(400, "password required"); }

    // Avoid duplicate users
    existing = redis_get("user:" + username);
    if existing != null { return json_err(409, "username already taken"); }

    // Store user (hash the password in a real app)
    redis_set("user:" + username, to_json({"username": username, "password": password}));

    // Issue token
    now   = 1700000000;          // use a real timestamp builtin if available
    token = make_token(username, now);
    redis_setex("session:" + token, TOKEN_TTL, to_json({"username": username}));

    return json_ok({"token": token, "username": username, "expires_in": TOKEN_TTL});
}

def handle_login(req) {
    body     = from_json(req["body"]);
    username = body["username"];
    password = body["password"];

    user_json = redis_get("user:" + username);
    if user_json == null { return json_err(401, "invalid credentials"); }

    user = from_json(user_json);
    if user["password"] != password { return json_err(401, "invalid credentials"); }

    now   = 1700000000;
    token = make_token(username, now);
    redis_setex("session:" + token, TOKEN_TTL, to_json({"username": username}));

    return json_ok({"token": token, "username": username, "expires_in": TOKEN_TTL});
}

// Protected route — returns the caller's profile
def handle_profile(req) {
    session = require_auth(req);
    if session == null {
        return json_err(401, "unauthorized — provide a valid Bearer token");
    }
    return json_ok({"username": session["username"], "message": "welcome back!"});
}

// Admin-only route — extend require_auth to check a role field
def handle_admin(req) {
    session = require_auth(req);
    if session == null {
        return json_err(401, "unauthorized");
    }
    role = session["role"];
    if role != "admin" {
        return json_err(403, "forbidden — admin role required");
    }
    return json_ok({"message": "hello, admin"});
}

// ── Router ────────────────────────────────────────────────────

def router(req) {
    method = req["method"];
    path   = req["path"];

    if method == "OPTIONS" {
        return {"status": 204, "body": "", "content_type": "text/plain"};
    }
    if path == "/register" { if method == "POST" { return handle_register(req); } }
    if path == "/login"    { if method == "POST" { return handle_login(req); } }
    if path == "/profile"  { if method == "GET"  { return handle_profile(req); } }
    if path == "/admin"    { if method == "GET"  { return handle_admin(req); } }
    return json_err(404, "not found");
}

// ── Boot ──────────────────────────────────────────────────────

redis_connect("redis://127.0.0.1:6379");
print("Auth demo running on http://localhost:8080");
serve(8080, router);
```

---

## Recipe 2: Pagination — paginated list and DB results

Pagination keeps response sizes predictable. Read `page` and `per_page` from the query string, convert them to SQL `LIMIT` / `OFFSET`, and always return the standard envelope `{ data, page, per_page, total, pages }` so clients can render navigation controls without extra requests.

**When to use:** Any endpoint that returns a variable-length collection — products, users, orders, search results, etc.

```unilang
// pagination_demo.uniL — paginate a SQLite result set
// Run: unilang run pagination_demo.uniL

// ── DB setup ──────────────────────────────────────────────────

def init_db() {
    db_connect("sqlite://pagination_demo.db");
    db_exec("CREATE TABLE IF NOT EXISTS items (id INTEGER PRIMARY KEY AUTOINCREMENT, name TEXT NOT NULL, category TEXT)", []);
    count_rows = db_query("SELECT COUNT(*) as n FROM items", []);
    if count_rows[0]["n"] > 0 { return; }
    i = 1;
    while i <= 95 {
        db_exec("INSERT INTO items (name, category) VALUES (?, ?)",
                ["Item " + str(i), "cat-" + str(i % 5)]);
        i = i + 1;
    }
    print("Seeded 95 items");
}

// ── Helpers ───────────────────────────────────────────────────

def get_query_param(query, key) {
    if query == "" { return ""; }
    if query == null { return ""; }
    parts = split(query, "&");
    i = 0;
    while i < len(parts) {
        kv = split(parts[i], "=");
        i = i + 1;
        if len(kv) >= 2 {
            if kv[0] == key { return kv[1]; }
        }
    }
    return "";
}

def parse_int_param(val, default_val) {
    if val == ""   { return default_val; }
    if val == null { return default_val; }
    n = int(val);
    if n < 1 { return default_val; }
    return n;
}

def ceil_div(a, b) {
    result = a / b;
    if a > result * b { result = result + 1; }
    return result;
}

def json_ok(data) {
    return {"status": 200, "body": to_json(data), "content_type": "application/json"};
}

def json_err(code, msg) {
    return {"status": code, "body": to_json({"error": msg}), "content_type": "application/json"};
}

// ── Core pagination helper ────────────────────────────────────
// Returns the standard { data, page, per_page, total, pages } envelope.
// Works with any list; also demonstrates SQL LIMIT/OFFSET variant below.

def paginate_list(all_items, page, per_page) {
    total  = len(all_items);
    pages  = ceil_div(total, per_page);
    start  = (page - 1) * per_page;
    end    = start + per_page;
    if start >= total { start = total; }
    if end   > total  { end   = total; }
    slice  = [];
    i = start;
    while i < end {
        slice.append(all_items[i]);
        i = i + 1;
    }
    return {"data": slice, "page": page, "per_page": per_page, "total": total, "pages": pages};
}

// ── Handlers ──────────────────────────────────────────────────

// In-memory pagination (useful for small result sets already in RAM)
def handle_list_memory(req) {
    query    = req["query"];
    page     = parse_int_param(get_query_param(query, "page"),     1);
    per_page = parse_int_param(get_query_param(query, "per_page"), 10);
    category = get_query_param(query, "category");

    // Load and filter
    all_rows = db_query("SELECT * FROM items ORDER BY id", []);
    filtered = [];
    i = 0;
    while i < len(all_rows) {
        r = all_rows[i];
        i = i + 1;
        if category != "" {
            if r["category"] != category { continue; }
        }
        filtered.append(r);
    }

    result = paginate_list(filtered, page, per_page);
    return json_ok(result);
}

// SQL LIMIT/OFFSET pagination (efficient for large tables)
def handle_list_sql(req) {
    query    = req["query"];
    page     = parse_int_param(get_query_param(query, "page"),     1);
    per_page = parse_int_param(get_query_param(query, "per_page"), 10);

    // Clamp per_page to a safe maximum
    if per_page > 100 { per_page = 100; }

    offset = (page - 1) * per_page;

    count_rows = db_query("SELECT COUNT(*) as n FROM items", []);
    total      = count_rows[0]["n"];
    pages      = ceil_div(total, per_page);

    data = db_query("SELECT * FROM items ORDER BY id LIMIT ? OFFSET ?",
                    [per_page, offset]);

    return json_ok({"data": data, "page": page, "per_page": per_page, "total": total, "pages": pages});
}

// ── Router ────────────────────────────────────────────────────

def router(req) {
    method = req["method"];
    path   = req["path"];
    if method == "OPTIONS" {
        return {"status": 204, "body": "", "content_type": "text/plain"};
    }
    // ?page=1&per_page=10&category=cat-2
    if path == "/items"     { if method == "GET" { return handle_list_memory(req); } }
    // ?page=1&per_page=10
    if path == "/items/sql" { if method == "GET" { return handle_list_sql(req); } }
    return {"status": 404, "body": to_json({"error": "not found"}), "content_type": "application/json"};
}

// ── Boot ──────────────────────────────────────────────────────

init_db();
print("Pagination demo on http://localhost:8080");
print("Try: /items?page=2&per_page=5");
print("Try: /items/sql?page=3&per_page=10");
serve(8080, router);
```

---

## Recipe 3: Caching with Redis — cache-aside pattern

Cache-aside (lazy loading) keeps your database free from repeated reads of the same data. On every request, check Redis first; on a cache miss, query the DB, write the result to Redis with a TTL, and return it. On the next request the cache is warm and the DB is never touched.

**When to use:** Read-heavy endpoints where the underlying data changes infrequently — product pages, user profiles, category lists, configuration. Combine with cache invalidation (`redis_del`) on writes.

```unilang
// cache_aside_demo.uniL — cache-aside with Redis + SQLite
// Run: unilang run cache_aside_demo.uniL

CACHE_TTL = 300;   // 5 minutes

// ── DB setup ──────────────────────────────────────────────────

def init_db() {
    db_connect("sqlite://cache_demo.db");
    db_exec("CREATE TABLE IF NOT EXISTS products (id TEXT PRIMARY KEY, name TEXT, price REAL, stock INTEGER)", []);
    existing = db_query("SELECT COUNT(*) as n FROM products", []);
    if existing[0]["n"] > 0 { return; }
    db_exec("INSERT INTO products VALUES ('P1','Widget Alpha',  9.99,  50)", []);
    db_exec("INSERT INTO products VALUES ('P2','Widget Beta',  19.99,  25)", []);
    db_exec("INSERT INTO products VALUES ('P3','Widget Gamma', 29.99,  10)", []);
    print("Seeded product table");
}

// ── Helpers ───────────────────────────────────────────────────

def json_ok(data) {
    return {"status": 200, "body": to_json(data), "content_type": "application/json"};
}

def json_err(code, msg) {
    return {"status": code, "body": to_json({"error": msg}), "content_type": "application/json"};
}

// ── Cache-aside read ──────────────────────────────────────────

def get_product(product_id) {
    cache_key = "product:" + product_id;

    // 1. Check Redis
    cached = redis_get(cache_key);
    if cached != null {
        product = from_json(cached);
        product["_source"] = "cache";
        return product;
    }

    // 2. Cache miss — hit the DB
    rows = db_query("SELECT * FROM products WHERE id = ?", [product_id]);
    if len(rows) == 0 { return null; }

    product = rows[0];
    product["_source"] = "db";

    // 3. Populate the cache for next time
    redis_setex(cache_key, CACHE_TTL, to_json(rows[0]));

    return product;
}

// ── Cache invalidation on write ───────────────────────────────

def update_product_price(product_id, new_price) {
    db_exec("UPDATE products SET price = ? WHERE id = ?", [new_price, product_id]);
    // Invalidate so the next read fetches fresh data
    redis_del("product:" + product_id);
}

// ── Handlers ──────────────────────────────────────────────────

def handle_get_product(req, product_id) {
    product = get_product(product_id);
    if product == null {
        return json_err(404, "product not found");
    }
    return json_ok(product);
}

def handle_update_price(req, product_id) {
    body      = from_json(req["body"]);
    new_price = body["price"];
    if new_price == null { return json_err(400, "price required"); }
    update_product_price(product_id, new_price);
    return json_ok({"message": "price updated, cache invalidated", "id": product_id});
}

def handle_cache_stats(req) {
    ttl_p1 = redis_ttl("product:P1");
    ttl_p2 = redis_ttl("product:P2");
    ttl_p3 = redis_ttl("product:P3");
    return json_ok({
        "product:P1": {"ttl_seconds": ttl_p1},
        "product:P2": {"ttl_seconds": ttl_p2},
        "product:P3": {"ttl_seconds": ttl_p3}
    });
}

// ── Router ────────────────────────────────────────────────────

def router(req) {
    method = req["method"];
    path   = req["path"];

    if method == "OPTIONS" {
        return {"status": 204, "body": "", "content_type": "text/plain"};
    }

    parts  = split(path, "/");
    nparts = len(parts);

    // GET  /products/:id
    // PUT  /products/:id/price
    if nparts >= 3 {
        if parts[1] == "products" {
            product_id = parts[2];
            if nparts == 3 {
                if method == "GET" { return handle_get_product(req, product_id); }
            }
            if nparts == 4 {
                if parts[3] == "price" {
                    if method == "PUT" { return handle_update_price(req, product_id); }
                }
            }
        }
    }

    if path == "/cache/stats" {
        if method == "GET" { return handle_cache_stats(req); }
    }

    return json_err(404, "not found");
}

// ── Boot ──────────────────────────────────────────────────────

init_db();
redis_connect("redis://127.0.0.1:6379");
print("Cache-aside demo on http://localhost:8080");
print("GET  /products/P1        — first call hits DB, subsequent calls hit cache");
print("PUT  /products/P1/price  — updates DB and invalidates cache");
print("GET  /cache/stats        — shows TTL of each cached key");
serve(8080, router);
```

---

## Recipe 4: Rate Limiting — per-IP sliding window

Per-IP rate limiting uses two Redis operations: `redis_incr` to count requests and `redis_expire` to set a window. Because `EXPIRE` is only set when the counter is first created, the window is a fixed interval that resets cleanly — a simple and effective approach for most APIs.

**When to use:** Public endpoints that must be protected against brute-force, scraping, or denial-of-service. Tune `RATE_LIMIT` and `RATE_WINDOW` to match your SLA.

```unilang
// rate_limit_demo.uniL — per-IP rate limiting with Redis INCR+EXPIRE
// Run: unilang run rate_limit_demo.uniL

RATE_LIMIT  = 10;   // maximum requests per window
RATE_WINDOW = 60;   // window size in seconds

// ── Helpers ───────────────────────────────────────────────────

def json_ok(data) {
    return {"status": 200, "body": to_json(data), "content_type": "application/json"};
}

def json_err(code, msg) {
    return {"status": code, "body": to_json({"error": msg, "code": code}), "content_type": "application/json"};
}

def get_client_ip(req) {
    // Honour X-Forwarded-For when behind a proxy
    forwarded = req["headers"]["x-forwarded-for"];
    if forwarded != null {
        if forwarded != "" { return forwarded; }
    }
    ip = req["headers"]["x-real-ip"];
    if ip != null {
        if ip != "" { return ip; }
    }
    return "unknown";
}

// ── Rate-limit check ──────────────────────────────────────────
// Returns a dict: { allowed: bool, count: int, limit: int, reset_in: int }

def check_rate_limit(client_ip) {
    key   = "rl:" + client_ip;
    count = redis_incr(key);       // atomically increment; creates key at 0 first

    // Set the expiry only on the first request in this window
    if count == 1 {
        redis_expire(key, RATE_WINDOW);
    }

    ttl = redis_ttl(key);
    allowed = true;
    if count > RATE_LIMIT { allowed = false; }

    return {
        "allowed":   allowed,
        "count":     count,
        "limit":     RATE_LIMIT,
        "reset_in":  ttl
    };
}

// ── Rate-limit middleware wrapper ─────────────────────────────
// Call this at the top of any handler.  Returns null when the
// request is allowed, or a ready-made 429 response when blocked.

def rate_limit_check(req) {
    ip     = get_client_ip(req);
    result = check_rate_limit(ip);
    if result["allowed"] == false {
        resp = json_err(429, "rate limit exceeded — try again in " + str(result["reset_in"]) + "s");
        // Attach standard rate-limit headers (informational)
        resp["headers"] = {
            "X-RateLimit-Limit":     str(result["limit"]),
            "X-RateLimit-Remaining": "0",
            "Retry-After":           str(result["reset_in"])
        };
        return resp;
    }
    return null;
}

// ── Handlers ──────────────────────────────────────────────────

def handle_search(req) {
    // Apply rate limiting before any real work
    blocked = rate_limit_check(req);
    if blocked != null { return blocked; }

    ip    = get_client_ip(req);
    key   = "rl:" + ip;
    count = redis_get(key);
    ttl   = redis_ttl(key);

    return json_ok({
        "message":   "search results here",
        "client_ip": ip,
        "requests_this_window": int(count),
        "window_resets_in":     ttl
    });
}

def handle_status(req) {
    ip     = get_client_ip(req);
    key    = "rl:" + ip;
    count  = redis_get(key);
    ttl    = redis_ttl(key);

    used = 0;
    if count != null { used = int(count); }
    remaining = RATE_LIMIT - used;
    if remaining < 0 { remaining = 0; }

    return json_ok({
        "client_ip":    ip,
        "limit":        RATE_LIMIT,
        "used":         used,
        "remaining":    remaining,
        "window_seconds": RATE_WINDOW,
        "reset_in":     ttl
    });
}

// ── Router ────────────────────────────────────────────────────

def router(req) {
    method = req["method"];
    path   = req["path"];
    if method == "OPTIONS" {
        return {"status": 204, "body": "", "content_type": "text/plain"};
    }
    if path == "/search" { if method == "GET" { return handle_search(req); } }
    if path == "/status" { if method == "GET" { return handle_status(req); } }
    return json_err(404, "not found");
}

// ── Boot ──────────────────────────────────────────────────────

redis_connect("redis://127.0.0.1:6379");
print("Rate-limit demo on http://localhost:8080");
print("GET /search — limited to " + str(RATE_LIMIT) + " req/" + str(RATE_WINDOW) + "s per IP");
print("GET /status — see your current usage");
serve(8080, router);
```

---

## Recipe 5: Background Jobs with Kafka — producer/consumer

Kafka lets you decouple slow or unreliable work from the HTTP request cycle. The HTTP handler writes a job message to a topic and returns immediately; a separate consumer loop reads the topic and processes each job. UniLang's in-memory Kafka driver (`kafka_produce` / `kafka_events`) works out of the box with no broker required.

**When to use:** Email sending, image resizing, report generation, webhook delivery, or any work that should not block the HTTP response.

```unilang
// kafka_jobs_demo.uniL — producer enqueues jobs, consumer processes them
// Run: unilang run kafka_jobs_demo.uniL

JOBS_TOPIC   = "background-jobs";
RESULTS_TOPIC = "job-results";

// ── Helpers ───────────────────────────────────────────────────

def json_ok(data) {
    return {"status": 200, "body": to_json(data), "content_type": "application/json"};
}

def json_err(code, msg) {
    return {"status": code, "body": to_json({"error": msg}), "content_type": "application/json"};
}

// ── Job producer ──────────────────────────────────────────────
// Enqueue a job and return a job ID to the caller immediately.

def enqueue_job(job_type, payload) {
    job_id  = "job-" + job_type + "-" + str(len(kafka_events()));
    message = to_json({
        "job_id":   job_id,
        "type":     job_type,
        "payload":  payload,
        "status":   "queued"
    });
    kafka_produce(JOBS_TOPIC, job_id, message);
    return job_id;
}

// ── Job consumer ──────────────────────────────────────────────
// Call process_pending_jobs() from a background thread or
// a periodic admin endpoint.  In a real deployment you would
// run this in a dedicated consumer process.

def process_job(job) {
    job_type = job["type"];
    payload  = job["payload"];
    result   = "unhandled";

    if job_type == "send_email" {
        // Simulate sending an email
        print("Sending email to: " + payload["to"]);
        result = "email sent to " + payload["to"];
    }
    if job_type == "resize_image" {
        // Simulate image processing
        print("Resizing image: " + payload["url"]);
        result = "resized " + payload["url"] + " to " + str(payload["width"]) + "px";
    }
    if job_type == "generate_report" {
        print("Generating report: " + payload["report_name"]);
        result = "report " + payload["report_name"] + " generated";
    }

    // Publish the result to a separate topic for auditing / polling
    kafka_produce(RESULTS_TOPIC, job["job_id"], to_json({
        "job_id":  job["job_id"],
        "type":    job_type,
        "status":  "completed",
        "result":  result
    }));

    return result;
}

def process_pending_jobs() {
    events    = kafka_events();
    processed = 0;
    i = 0;
    while i < len(events) {
        evt = events[i];
        i = i + 1;
        if evt["topic"] == JOBS_TOPIC {
            job = from_json(evt["value"]);
            process_job(job);
            processed = processed + 1;
        }
    }
    return processed;
}

// ── Handlers ──────────────────────────────────────────────────

def handle_submit_job(req) {
    body     = from_json(req["body"]);
    job_type = body["type"];
    payload  = body["payload"];

    if job_type == null { return json_err(400, "type required"); }
    if payload  == null { return json_err(400, "payload required"); }

    job_id = enqueue_job(job_type, payload);
    return json_ok({"job_id": job_id, "status": "queued", "message": "job enqueued successfully"});
}

// Admin endpoint: drain the queue and process all pending jobs
def handle_process_jobs(req) {
    count = process_pending_jobs();
    return json_ok({"processed": count, "message": str(count) + " jobs processed"});
}

// Show all events (jobs + results) — useful for debugging
def handle_list_events(req) {
    events = kafka_events();
    return json_ok({"events": events, "total": len(events)});
}

// ── Router ────────────────────────────────────────────────────

def router(req) {
    method = req["method"];
    path   = req["path"];
    if method == "OPTIONS" {
        return {"status": 204, "body": "", "content_type": "text/plain"};
    }
    if path == "/jobs" {
        if method == "POST" { return handle_submit_job(req); }
    }
    if path == "/jobs/process" {
        if method == "POST" { return handle_process_jobs(req); }
    }
    if path == "/events" {
        if method == "GET" { return handle_list_events(req); }
    }
    return json_err(404, "not found");
}

// ── Boot ──────────────────────────────────────────────────────

print("Kafka jobs demo on http://localhost:8080");
print("POST /jobs          — { type: send_email, payload: { to: ... } }");
print("POST /jobs/process  — drain and process the job queue");
print("GET  /events        — inspect the Kafka event log");
serve(8080, router);
```

---

## Recipe 6: Input Validation — required fields, types, lengths

Validate every field before touching the database. A reusable `validate` helper accumulates all errors in a single pass and returns a structured list, so the client sees every problem in one response rather than one error per round-trip.

**When to use:** Any `POST` or `PUT` handler that accepts a JSON body. Front-load validation before business logic to keep handler code clean.

```unilang
// validation_demo.uniL — structured input validation
// Run: unilang run validation_demo.uniL

// ── Validation helpers ────────────────────────────────────────

// Each check appends to the `errors` list and returns it.
// Call validate_required, validate_type, etc. in sequence, then
// check len(errors) == 0 before proceeding.

def validate_required(errors, data, field) {
    val = data[field];
    if val == null {
        errors.append({"field": field, "message": field + " is required"});
        return errors;
    }
    if val == "" {
        errors.append({"field": field, "message": field + " must not be empty"});
    }
    return errors;
}

def validate_min_length(errors, data, field, min_len) {
    val = data[field];
    if val == null { return errors; }
    if len(val) < min_len {
        errors.append({"field": field,
                        "message": field + " must be at least " + str(min_len) + " characters"});
    }
    return errors;
}

def validate_max_length(errors, data, field, max_len) {
    val = data[field];
    if val == null { return errors; }
    if len(val) > max_len {
        errors.append({"field": field,
                        "message": field + " must be at most " + str(max_len) + " characters"});
    }
    return errors;
}

def validate_is_number(errors, data, field) {
    val = data[field];
    if val == null { return errors; }
    // Attempt conversion; if it fails, flag the error
    converted = float(val);
    if converted == null {
        errors.append({"field": field, "message": field + " must be a number"});
    }
    return errors;
}

def validate_min_value(errors, data, field, min_val) {
    val = data[field];
    if val == null { return errors; }
    n = float(val);
    if n == null { return errors; }
    if n < min_val {
        errors.append({"field": field,
                        "message": field + " must be >= " + str(min_val)});
    }
    return errors;
}

def validate_max_value(errors, data, field, max_val) {
    val = data[field];
    if val == null { return errors; }
    n = float(val);
    if n == null { return errors; }
    if n > max_val {
        errors.append({"field": field,
                        "message": field + " must be <= " + str(max_val)});
    }
    return errors;
}

def validate_email(errors, data, field) {
    val = data[field];
    if val == null { return errors; }
    if val == ""   { return errors; }
    if contains(val, "@") == false {
        errors.append({"field": field, "message": field + " must be a valid email address"});
    }
    return errors;
}

def validate_one_of(errors, data, field, allowed) {
    val = data[field];
    if val == null { return errors; }
    found = false;
    i = 0;
    while i < len(allowed) {
        if allowed[i] == val { found = true; }
        i = i + 1;
    }
    if found == false {
        errors.append({"field": field,
                        "message": field + " must be one of: " + join(allowed, ", ")});
    }
    return errors;
}

// ── Helpers ───────────────────────────────────────────────────

def json_ok(data) {
    return {"status": 200, "body": to_json(data), "content_type": "application/json"};
}

def json_err(code, msg) {
    return {"status": code, "body": to_json({"error": msg}), "content_type": "application/json"};
}

def validation_error(errors) {
    return {
        "status": 422,
        "body": to_json({"error": "validation failed", "code": 422, "details": errors}),
        "content_type": "application/json"
    };
}

// ── Handlers ──────────────────────────────────────────────────

def handle_create_user(req) {
    if req["body"] == "" { return json_err(400, "request body is empty"); }
    if req["body"] == null { return json_err(400, "request body is empty"); }

    data = from_json(req["body"]);
    if data == null { return json_err(400, "invalid JSON"); }

    errors = [];

    // Required fields
    errors = validate_required(errors, data, "username");
    errors = validate_required(errors, data, "email");
    errors = validate_required(errors, data, "password");
    errors = validate_required(errors, data, "role");

    // Length constraints
    errors = validate_min_length(errors, data, "username", 3);
    errors = validate_max_length(errors, data, "username", 32);
    errors = validate_min_length(errors, data, "password", 8);
    errors = validate_max_length(errors, data, "password", 128);

    // Format checks
    errors = validate_email(errors, data, "email");

    // Enum check
    errors = validate_one_of(errors, data, "role", ["user", "editor", "admin"]);

    if len(errors) > 0 { return validation_error(errors); }

    // All good — proceed with business logic
    return json_ok({
        "message":  "user created",
        "username": data["username"],
        "email":    data["email"],
        "role":     data["role"]
    });
}

def handle_create_product(req) {
    if req["body"] == "" { return json_err(400, "request body is empty"); }

    data   = from_json(req["body"]);
    errors = [];

    errors = validate_required(errors, data, "name");
    errors = validate_required(errors, data, "price");
    errors = validate_required(errors, data, "stock");

    errors = validate_min_length(errors, data, "name", 2);
    errors = validate_max_length(errors, data, "name", 200);

    errors = validate_is_number(errors, data, "price");
    errors = validate_min_value(errors, data, "price", 0.01);
    errors = validate_max_value(errors, data, "price", 999999.99);

    errors = validate_is_number(errors, data, "stock");
    errors = validate_min_value(errors, data, "stock", 0);

    if len(errors) > 0 { return validation_error(errors); }

    return json_ok({"message": "product created", "name": data["name"], "price": float(data["price"])});
}

// ── Router ────────────────────────────────────────────────────

def router(req) {
    method = req["method"];
    path   = req["path"];
    if method == "OPTIONS" {
        return {"status": 204, "body": "", "content_type": "text/plain"};
    }
    if path == "/users"    { if method == "POST" { return handle_create_user(req); } }
    if path == "/products" { if method == "POST" { return handle_create_product(req); } }
    return json_err(404, "not found");
}

// ── Boot ──────────────────────────────────────────────────────

print("Validation demo on http://localhost:8080");
print("POST /users    — { username, email, password, role }");
print("POST /products — { name, price, stock }");
serve(8080, router);
```

---

## Recipe 7: Error Handling Middleware — consistent JSON errors

Consistent error envelopes make clients easier to write. A top-level `safe_router` wrapper catches any uncaught condition from the inner router and always returns `{ error, code, message }` — never an empty body or an HTML error page.

**When to use:** Every production server. Wrap your router once at the entry point and every unhandled case gets a well-formed JSON response automatically.

```unilang
// error_middleware_demo.uniL — uniform { error, code, message } JSON errors
// Run: unilang run error_middleware_demo.uniL

// ── Standard error constructors ───────────────────────────────

def err_400(message) {
    return {"status": 400, "body": to_json({"error": "bad_request",    "code": 400, "message": message}), "content_type": "application/json"};
}

def err_401(message) {
    return {"status": 401, "body": to_json({"error": "unauthorized",   "code": 401, "message": message}), "content_type": "application/json"};
}

def err_403(message) {
    return {"status": 403, "body": to_json({"error": "forbidden",      "code": 403, "message": message}), "content_type": "application/json"};
}

def err_404(message) {
    return {"status": 404, "body": to_json({"error": "not_found",      "code": 404, "message": message}), "content_type": "application/json"};
}

def err_405(message) {
    return {"status": 405, "body": to_json({"error": "method_not_allowed", "code": 405, "message": message}), "content_type": "application/json"};
}

def err_409(message) {
    return {"status": 409, "body": to_json({"error": "conflict",       "code": 409, "message": message}), "content_type": "application/json"};
}

def err_422(message, details) {
    return {"status": 422, "body": to_json({"error": "unprocessable",  "code": 422, "message": message, "details": details}), "content_type": "application/json"};
}

def err_500(message) {
    return {"status": 500, "body": to_json({"error": "internal_error", "code": 500, "message": message}), "content_type": "application/json"};
}

def json_ok(data) {
    return {"status": 200, "body": to_json(data), "content_type": "application/json"};
}

// ── Request logger ────────────────────────────────────────────

def log_request(req, resp) {
    method = req["method"];
    path   = req["path"];
    status = resp["status"];
    print("[" + method + "] " + path + " -> " + str(status));
}

// ── Inner router (your actual business logic) ─────────────────

def inner_router(req) {
    method = req["method"];
    path   = req["path"];

    if method == "OPTIONS" {
        return {"status": 204, "body": "", "content_type": "text/plain"};
    }

    // Demonstrate each error type
    if path == "/ok"    { if method == "GET" { return json_ok({"message": "everything is fine"}); } }
    if path == "/error" { if method == "GET" { return err_400("bad input example"); } }
    if path == "/auth"  { if method == "GET" { return err_401("please log in"); } }
    if path == "/admin" { if method == "GET" { return err_403("admins only"); } }
    if path == "/boom"  { if method == "GET" { return err_500("something went wrong internally"); } }

    // Only GET is supported on /items
    if path == "/items" {
        if method == "GET" {
            return json_ok({"items": ["apple", "banana", "cherry"]});
        }
        return err_405(method + " not allowed on /items");
    }

    // Unknown path
    return err_404("no route matches " + method + " " + path);
}

// ── Error-handling middleware wrapper ─────────────────────────
// Wraps inner_router: ensures a valid response is always returned
// and appends standard headers to every response.

def safe_router(req) {
    resp = null;

    // Guard: validate that the request object has required fields
    if req == null {
        resp = err_500("request object is null");
    }

    if resp == null {
        if req["method"] == null {
            resp = err_400("missing method in request");
        }
    }

    // Dispatch to inner router
    if resp == null {
        resp = inner_router(req);
    }

    // Fallback: inner_router returned null (should not happen, but be safe)
    if resp == null {
        resp = err_500("handler returned no response");
    }

    // Ensure content_type is always set
    if resp["content_type"] == null {
        resp["content_type"] = "application/json";
    }

    // Log every request/response
    log_request(req, resp);

    return resp;
}

// ── Boot ──────────────────────────────────────────────────────

print("Error middleware demo on http://localhost:8080");
print("GET /ok     — 200 success");
print("GET /error  — 400 bad request");
print("GET /auth   — 401 unauthorized");
print("GET /admin  — 403 forbidden");
print("GET /boom   — 500 internal error");
print("GET /items  — 200  |  POST /items — 405 method not allowed");
print("GET /xyz    — 404 not found");
serve(8080, safe_router);
```

---

## Recipe 8: Environment-Based Config — env vars with defaults

Hard-coded secrets and ports belong in environment variables, not source files. A central `load_config` function reads each variable with a sensible default, making the application behave identically in development (defaults) and production (env overrides) without code changes.

**When to use:** Any server you intend to deploy beyond a single laptop. Keep all tuneable values — DB paths, Redis URLs, secrets, ports, feature flags — in one config dict loaded at startup.

```unilang
// env_config_demo.uniL — env vars with defaults
// Run: unilang run env_config_demo.uniL
// Override at runtime:
//   APP_PORT=9090 DB_PATH=./prod.db APP_SECRET=s3cr3t unilang run env_config_demo.uniL

// ── Config loader ─────────────────────────────────────────────
// env_get(key, default) reads an environment variable; returns
// default when the variable is unset or empty.

def env_get(key, default_val) {
    val = get_env(key);
    if val == null { return default_val; }
    if val == ""   { return default_val; }
    return val;
}

def env_int(key, default_val) {
    val = get_env(key);
    if val == null { return default_val; }
    if val == ""   { return default_val; }
    n = int(val);
    if n == null { return default_val; }
    return n;
}

def load_config() {
    return {
        // Server
        "port":         env_int("APP_PORT",      8080),
        "host":         env_get("APP_HOST",       "0.0.0.0"),

        // Database
        "db_path":      env_get("DB_PATH",        "sqlite://app.db"),

        // Redis
        "redis_url":    env_get("REDIS_URL",      "redis://127.0.0.1:6379"),
        "cache_ttl":    env_int("CACHE_TTL",      300),

        // Security
        "app_secret":   env_get("APP_SECRET",     "dev-secret-change-in-production"),
        "token_ttl":    env_int("TOKEN_TTL",      3600),

        // Feature flags
        "enable_cache": env_get("ENABLE_CACHE",   "true"),
        "debug":        env_get("DEBUG",          "false"),

        // Kafka
        "kafka_broker": env_get("KAFKA_BROKER",   "localhost:9092"),
        "jobs_topic":   env_get("KAFKA_JOBS_TOPIC","background-jobs")
    };
}

// ── Startup ───────────────────────────────────────────────────

CONFIG = load_config();

// Warn loudly if the default secret is still in use
if CONFIG["app_secret"] == "dev-secret-change-in-production" {
    print("WARNING: APP_SECRET is using the insecure development default.");
    print("         Set APP_SECRET in your environment before deploying.");
}

if CONFIG["debug"] == "true" {
    print("DEBUG mode enabled — config dump:");
    safe_config = {
        "port":         CONFIG["port"],
        "host":         CONFIG["host"],
        "db_path":      CONFIG["db_path"],
        "redis_url":    CONFIG["redis_url"],
        "cache_ttl":    CONFIG["cache_ttl"],
        "token_ttl":    CONFIG["token_ttl"],
        "enable_cache": CONFIG["enable_cache"],
        "debug":        CONFIG["debug"],
        "kafka_broker": CONFIG["kafka_broker"],
        "jobs_topic":   CONFIG["jobs_topic"]
        // app_secret intentionally omitted from debug output
    };
    print(to_json(safe_config));
}

// ── DB and Redis init using config values ─────────────────────

db_connect(CONFIG["db_path"]);
db_exec("CREATE TABLE IF NOT EXISTS notes (id INTEGER PRIMARY KEY AUTOINCREMENT, body TEXT, created_at TEXT DEFAULT CURRENT_TIMESTAMP)", []);

redis_connect(CONFIG["redis_url"]);

// ── Helpers ───────────────────────────────────────────────────

def json_ok(data) {
    return {"status": 200, "body": to_json(data), "content_type": "application/json"};
}

def json_err(code, msg) {
    return {"status": code, "body": to_json({"error": msg}), "content_type": "application/json"};
}

// ── Handlers ──────────────────────────────────────────────────

def handle_config(req) {
    // Expose non-sensitive config for health checks / dashboards
    return json_ok({
        "port":         CONFIG["port"],
        "db_path":      CONFIG["db_path"],
        "redis_url":    CONFIG["redis_url"],
        "cache_ttl":    CONFIG["cache_ttl"],
        "token_ttl":    CONFIG["token_ttl"],
        "enable_cache": CONFIG["enable_cache"],
        "debug":        CONFIG["debug"],
        "jobs_topic":   CONFIG["jobs_topic"]
    });
}

def handle_create_note(req) {
    body = from_json(req["body"]);
    if body == null { return json_err(400, "invalid JSON"); }
    note_body = body["body"];
    if note_body == null { return json_err(400, "body field required"); }
    if note_body == ""   { return json_err(400, "body field required"); }

    db_exec("INSERT INTO notes (body) VALUES (?)", [note_body]);

    // Cache is controlled by config flag
    if CONFIG["enable_cache"] == "true" {
        redis_setex("notes:latest", CONFIG["cache_ttl"], note_body);
    }

    return json_ok({"message": "note saved", "body": note_body});
}

def handle_get_latest(req) {
    if CONFIG["enable_cache"] == "true" {
        cached = redis_get("notes:latest");
        if cached != null {
            return json_ok({"body": cached, "source": "cache"});
        }
    }
    rows = db_query("SELECT * FROM notes ORDER BY id DESC LIMIT 1", []);
    if len(rows) == 0 { return json_err(404, "no notes yet"); }
    return json_ok({"body": rows[0]["body"], "source": "db"});
}

// ── Router ────────────────────────────────────────────────────

def router(req) {
    method = req["method"];
    path   = req["path"];
    if method == "OPTIONS" {
        return {"status": 204, "body": "", "content_type": "text/plain"};
    }
    if path == "/config" { if method == "GET"  { return handle_config(req); } }
    if path == "/notes"  {
        if method == "POST" { return handle_create_note(req); }
        if method == "GET"  { return handle_get_latest(req); }
    }
    return json_err(404, "not found");
}

// ── Boot ──────────────────────────────────────────────────────

print("Env-config demo on http://localhost:" + str(CONFIG["port"]));
print("GET  /config — inspect active configuration");
print("POST /notes  — { body: ... }");
print("GET  /notes  — fetch latest note (cache-aware)");
serve(CONFIG["port"], router);
```

---

## Putting It All Together

The recipes are designed to compose. A realistic production server typically uses all eight patterns at once:

```
request
  │
  └── safe_router (Recipe 7 — error middleware)
        │
        ├── rate_limit_check (Recipe 4)
        ├── require_auth     (Recipe 1)
        ├── validate_*       (Recipe 6)
        │
        └── handler
              ├── get_product / cache-aside (Recipe 3)
              ├── paginate_list / SQL LIMIT  (Recipe 2)
              └── enqueue_job               (Recipe 5)
                    │
                    └── Kafka consumer (Recipe 5)

CONFIG loaded once at startup (Recipe 8)
```

See the [SHYNX e-commerce example](../../examples/ecommerce/backend/server.uniL) for a full server that combines SQLite, Redis, Kafka, pagination, and caching in a single file.
