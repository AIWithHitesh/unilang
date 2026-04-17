# Python → UniLang Migration Guide

**Version:** 1.0.0
**Last Updated:** 2026-04-17

---

## Table of Contents

1. [Introduction](#1-introduction)
2. [What Works Unchanged](#2-what-works-unchanged)
3. [What Differs](#3-what-differs)
4. [New Capabilities from Java Syntax](#4-new-capabilities-from-java-syntax)
5. [Python Stdlib → UniLang Builtins](#5-python-stdlib--unilang-builtins)
6. [Common Migration Patterns](#6-common-migration-patterns)
7. [Complete Migration Example](#7-complete-migration-example)

---

## 1. Introduction

UniLang is a unified language that merges Python and Java syntax into a single runtime. If you are a Python developer, the most important thing to know is: **most of your Python code will run in UniLang without any changes.** The `.uniL` runtime understands Python syntax natively — `def`, `class`, `for`, `if`, f-strings, list comprehensions, decorators, and everything else you rely on day-to-day.

This guide focuses on three areas:

- **What already works** — Python constructs you can paste directly into a `.uniL` file and run.
- **What differs** — places where UniLang provides a builtin that replaces a Python stdlib import.
- **What is new** — Java-style syntax and UniLang-specific builtins that you can adopt incrementally.

The migration is almost always a simplification: UniLang replaces many multi-step `import X; X.do_thing()` patterns with single, globally available builtin functions. You drop a large block of `import` statements at the top of your file and call the builtins directly.

---

## 2. What Works Unchanged

The following Python constructs are valid UniLang syntax with identical semantics. No changes required.

### Functions

```python
def add(a, b):
    return a + b

def greet(name, prefix="Hello"):
    return f"{prefix}, {name}!"
```

### *args and **kwargs

```python
def log(*args, **kwargs):
    for arg in args:
        print(arg)
    for key, value in kwargs.items():
        print(f"{key}={value}")
```

### Classes (Python-style)

```python
class Animal:
    def __init__(self, name):
        self.name = name

    def speak(self):
        return f"{self.name} makes a sound"

class Dog(Animal):
    def speak(self):
        return f"{self.name} barks"
```

### Control Flow

```python
for item in collection:
    if item > 0:
        print(item)
    elif item == 0:
        print("zero")
    else:
        print("negative")
```

### List Comprehensions

```python
squares = [x * x for x in range(10) if x % 2 == 0]
lookup = {k: v for k, v in pairs}
```

### Exception Handling

```python
try:
    result = risky_operation()
except ValueError as e:
    print(f"Value error: {e}")
except Exception as e:
    print(f"Unexpected: {e}")
finally:
    cleanup()
```

### Decorators

```python
def require_auth(func):
    def wrapper(*args, **kwargs):
        if not is_authenticated():
            return {"error": "unauthorized"}, 401
        return func(*args, **kwargs)
    return wrapper

@require_auth
def secret_endpoint():
    return {"data": "classified"}
```

### f-strings

```python
name = "World"
greeting = f"Hello, {name}!"
debug = f"Value is {1 + 1} and type is {type(name).__name__}"
```

### Imports (UniLang modules)

The `import` statement works for UniLang modules and packages located on the UniLang module path:

```python
import my_module
from my_package import my_function
```

> Note: Standard Python `import` for CPython stdlib modules (`os`, `sys`, `json`, etc.) does not resolve at runtime — use UniLang builtins instead. See [Section 3](#3-what-differs).

---

## 3. What Differs

UniLang provides a set of globally available builtin functions that replace the most common Python stdlib imports. You no longer need to import these modules — the functions are available everywhere.

### Side-by-Side Reference Table

| Python (stdlib) | UniLang builtin | Notes |
|---|---|---|
| `print(x)` | `print(x)` | Identical — no change needed |
| `len(x)` | `len(x)` | Identical |
| `str(x)` | `str(x)` | Identical |
| `int(x)` | `int(x)` | Identical |
| `float(x)` | `float(x)` | Identical |
| `import json; json.loads(s)` | `from_json(s)` | Parses JSON string → object |
| `import json; json.dumps(obj)` | `to_json(obj)` | Serialises object → JSON string |
| `import os; os.environ.get("KEY")` | `env_get("KEY")` | Read environment variable |
| `import requests; requests.get(url)` | `http_get(url)` | HTTP GET, returns response object |
| `import requests; requests.post(url, json=body)` | `http_post(url, body)` | HTTP POST with JSON body |
| `open(f).read()` | `read_file(f)` | Reads entire file as string |
| `open(f, "w").write(data)` | `write_file(f, data)` | Writes string to file |
| `import sqlite3` | `db_connect(path)` | Returns a DB connection handle |
| `cursor.execute(sql, params)` | `db_query(conn, sql, params)` | Execute SQL, returns rows |
| `import redis; redis.Redis(...)` | `redis_connect(host, port)` | Returns a Redis handle |
| `r.get(key)` | `redis_get(handle, key)` | Redis GET |
| `r.set(key, value)` | `redis_set(handle, key, value)` | Redis SET |
| `import flask; @app.route(...)` | `serve(port, routes)` | Declarative HTTP server |
| `import time; time.time()` | `now()` | Current Unix timestamp (float) |
| `import random; random.random()` | `random()` | Random float in [0.0, 1.0) |
| `import random; random.randint(a, b)` | `rand_int(a, b)` | Random integer in [a, b] |
| `import uuid; uuid.uuid4()` | `uuid()` | Generate a UUID string |
| `import hashlib; hashlib.sha256(s)` | `hash_sha256(s)` | SHA-256 hex digest |
| `import base64; base64.b64encode(b)` | `b64_encode(s)` | Base64 encode |
| `import base64; base64.b64decode(b)` | `b64_decode(s)` | Base64 decode |

### Key Behavioural Notes

**`from_json` / `to_json`** — These replace the entire `json` module for typical use:

```python
# Python
import json
data = json.loads(response.text)
payload = json.dumps({"status": "ok"})

# UniLang
data = from_json(response.body)
payload = to_json({"status": "ok"})
```

**`env_get`** — Returns `None` (not raises) when the variable is absent. Pass a default as the second argument:

```python
# Python
import os
port = int(os.environ.get("PORT", 8080))

# UniLang
port = int(env_get("PORT") or 8080)
```

**`http_get` / `http_post`** — Return a response object with `.status`, `.body`, and `.headers` attributes:

```python
# Python
import requests
resp = requests.get("https://api.example.com/data")
data = resp.json()

# UniLang
resp = http_get("https://api.example.com/data")
data = from_json(resp.body)
```

**`serve`** — Replaces Flask's application factory pattern with a single declarative call. Routes are a dict mapping `"METHOD /path"` to handler functions:

```python
# UniLang
def handle_index(req):
    return {"message": "hello"}

serve(8080, {
    "GET /": handle_index
})
```

---

## 4. New Capabilities from Java Syntax

Because UniLang is a unified Python+Java language, you can adopt Java syntax features incrementally alongside your existing Python code. None of these are required — but they are available if you want stronger typing or Java-style structure.

### Typed Variable Declarations

```java
// Java-style typed variables — valid in UniLang
int count = 0;
String name = "Alice";
double price = 9.99;
List<String> tags = new ArrayList<>();
```

These can sit in the same file as Python-style variables:

```python
# Python-style on the same line is also fine
count = 0
name = "Alice"
```

### Java-style Classes with Braces

```java
public class UserService {
    private String dbPath;

    public UserService(String dbPath) {
        this.dbPath = dbPath;
    }

    public List<Map> getUsers() {
        conn = db_connect(this.dbPath)
        return db_query(conn, "SELECT * FROM users", [])
    }
}
```

### try/catch (Java-style Exception Handling)

```java
try {
    result = db_query(conn, sql, params);
} catch (Exception e) {
    print(f"Query failed: {e}");
} finally {
    conn.close();
}
```

### Generics in Type Hints

```java
Map<String, Integer> scores = new HashMap<>();
List<String> names = new ArrayList<>();
```

### Access Modifiers

```java
public class Config {
    private String secret;
    public String appName;

    public Config(String appName, String secret) {
        this.appName = appName;
        this.secret = secret;
    }
}
```

### Mixing Python and Java in the Same File

UniLang is designed for this to be natural:

```python
// UniLang — mixing both styles freely

# Python-style utility function
def format_name(first, last):
    return f"{first} {last}"

# Java-style class with typed fields
public class Person {
    public String firstName;
    public String lastName;

    public Person(String firstName, String lastName) {
        this.firstName = firstName;
        this.lastName = lastName;
    }

    # Python-style method inside a Java-style class
    def full_name(self):
        return format_name(self.firstName, self.lastName)
}

p = Person("Ada", "Lovelace")
print(p.full_name())   # Ada Lovelace
```

---

## 5. Python Stdlib → UniLang Builtins

Comprehensive mapping of Python standard library modules to UniLang builtins.

### json

| Python | UniLang |
|---|---|
| `json.loads(s)` | `from_json(s)` |
| `json.dumps(obj)` | `to_json(obj)` |
| `json.dumps(obj, indent=2)` | `to_json_pretty(obj)` |

### os / sys

| Python | UniLang |
|---|---|
| `os.environ.get(key)` | `env_get(key)` |
| `os.environ.get(key, default)` | `env_get(key)` with `or default` |
| `os.path.exists(path)` | `file_exists(path)` |
| `os.getcwd()` | `cwd()` |
| `sys.exit(code)` | `exit(code)` |

### io / open

| Python | UniLang |
|---|---|
| `open(f).read()` | `read_file(f)` |
| `open(f, "w").write(data)` | `write_file(f, data)` |
| `open(f, "a").write(data)` | `append_file(f, data)` |

### requests / urllib

| Python | UniLang |
|---|---|
| `requests.get(url)` | `http_get(url)` |
| `requests.get(url, headers=h)` | `http_get(url, headers=h)` |
| `requests.post(url, json=body)` | `http_post(url, body)` |
| `requests.put(url, json=body)` | `http_put(url, body)` |
| `requests.delete(url)` | `http_delete(url)` |

### sqlite3

| Python | UniLang |
|---|---|
| `sqlite3.connect(path)` | `db_connect(path)` |
| `cursor.execute(sql, params)` | `db_query(conn, sql, params)` |
| `cursor.fetchall()` | (returned directly by `db_query`) |
| `conn.commit()` | `db_commit(conn)` |
| `conn.close()` | `db_close(conn)` |

### redis

| Python | UniLang |
|---|---|
| `redis.Redis(host, port)` | `redis_connect(host, port)` |
| `r.get(key)` | `redis_get(handle, key)` |
| `r.set(key, value)` | `redis_set(handle, key, value)` |
| `r.delete(key)` | `redis_del(handle, key)` |
| `r.exists(key)` | `redis_exists(handle, key)` |
| `r.expire(key, ttl)` | `redis_expire(handle, key, ttl)` |
| `r.lpush(key, value)` | `redis_lpush(handle, key, value)` |
| `r.rpop(key)` | `redis_rpop(handle, key)` |

### flask / http.server

| Python | UniLang |
|---|---|
| `Flask(__name__)` + `@app.route(...)` | `serve(port, routes)` |
| `request.json` | `req.body` (parsed automatically) |
| `request.args.get(k)` | `req.query[k]` |
| `request.headers.get(k)` | `req.headers[k]` |
| `jsonify(obj)` | return a dict directly |

### time / datetime

| Python | UniLang |
|---|---|
| `time.time()` | `now()` |
| `time.sleep(s)` | `sleep(s)` |
| `datetime.datetime.utcnow().isoformat()` | `now_iso()` |

### random

| Python | UniLang |
|---|---|
| `random.random()` | `random()` |
| `random.randint(a, b)` | `rand_int(a, b)` |
| `random.choice(seq)` | `rand_choice(seq)` |
| `random.shuffle(seq)` | `rand_shuffle(seq)` |

### hashlib / uuid / base64

| Python | UniLang |
|---|---|
| `hashlib.sha256(s.encode()).hexdigest()` | `hash_sha256(s)` |
| `hashlib.md5(s.encode()).hexdigest()` | `hash_md5(s)` |
| `str(uuid.uuid4())` | `uuid()` |
| `base64.b64encode(s.encode()).decode()` | `b64_encode(s)` |
| `base64.b64decode(s).decode()` | `b64_decode(s)` |

### logging

| Python | UniLang |
|---|---|
| `logging.info(msg)` | `log_info(msg)` |
| `logging.warning(msg)` | `log_warn(msg)` |
| `logging.error(msg)` | `log_error(msg)` |
| `logging.debug(msg)` | `log_debug(msg)` |

---

## 6. Common Migration Patterns

### Flask App → UniLang serve()

**Python (Flask)**

```python
from flask import Flask, request, jsonify
import sqlite3

app = Flask(__name__)
DB = "users.db"

@app.route("/users", methods=["GET"])
def list_users():
    conn = sqlite3.connect(DB)
    cursor = conn.cursor()
    cursor.execute("SELECT id, name FROM users")
    rows = [{"id": r[0], "name": r[1]} for r in cursor.fetchall()]
    conn.close()
    return jsonify(rows)

@app.route("/users", methods=["POST"])
def create_user():
    body = request.get_json()
    conn = sqlite3.connect(DB)
    conn.execute("INSERT INTO users (name) VALUES (?)", [body["name"]])
    conn.commit()
    conn.close()
    return jsonify({"status": "created"}), 201

if __name__ == "__main__":
    app.run(port=8080)
```

**UniLang**

```python
DB = "users.db"

def list_users(req):
    conn = db_connect(DB)
    rows = db_query(conn, "SELECT id, name FROM users", [])
    return [{"id": r[0], "name": r[1]} for r in rows]

def create_user(req):
    body = req.body
    conn = db_connect(DB)
    db_query(conn, "INSERT INTO users (name) VALUES (?)", [body["name"]])
    db_commit(conn)
    return {"status": "created"}

serve(8080, {
    "GET /users":  list_users,
    "POST /users": create_user
})
```

Note: 30 lines of Python with 4 imports becomes 17 lines of UniLang with zero imports.

---

### SQLAlchemy → db_connect / db_query

**Python (SQLAlchemy)**

```python
from sqlalchemy import create_engine, text

engine = create_engine("sqlite:///products.db")

def get_products():
    with engine.connect() as conn:
        result = conn.execute(text("SELECT * FROM products"))
        return [dict(row) for row in result.mappings()]

def add_product(name, price):
    with engine.connect() as conn:
        conn.execute(
            text("INSERT INTO products (name, price) VALUES (:name, :price)"),
            {"name": name, "price": price}
        )
        conn.commit()
```

**UniLang**

```python
conn = db_connect("products.db")

def get_products():
    rows = db_query(conn, "SELECT * FROM products", [])
    return rows

def add_product(name, price):
    db_query(conn, "INSERT INTO products (name, price) VALUES (?, ?)", [name, price])
    db_commit(conn)
```

---

### Pandas CSV Parsing → read_file + split

**Python (Pandas)**

```python
import pandas as pd

def load_sales(path):
    df = pd.read_csv(path)
    total = df["amount"].sum()
    return {"rows": len(df), "total": total}
```

**UniLang**

```python
def load_sales(path):
    raw = read_file(path)
    lines = raw.strip().split("\n")
    headers = lines[0].split(",")
    rows = [dict(zip(headers, line.split(","))) for line in lines[1:]]
    total = sum(float(r["amount"]) for r in rows)
    return {"rows": len(rows), "total": total}
```

For large or complex CSV work you can still call into a Java library via the `bridge` keyword — see the Language Spec section 13.

---

### Redis-py → redis_* Builtins

**Python (redis-py)**

```python
import redis
import json

r = redis.Redis(host="localhost", port=6379)

def cache_get(key):
    val = r.get(key)
    return json.loads(val) if val else None

def cache_set(key, obj, ttl=300):
    r.set(key, json.dumps(obj))
    r.expire(key, ttl)
```

**UniLang**

```python
r = redis_connect("localhost", 6379)

def cache_get(key):
    val = redis_get(r, key)
    return from_json(val) if val else None

def cache_set(key, obj, ttl=300):
    redis_set(r, key, to_json(obj))
    redis_expire(r, key, ttl)
```

---

## 7. Complete Migration Example

A real-world Python Flask REST service (product catalogue) migrated line-by-line to UniLang.

### Python Original (Flask, ~35 lines)

```python
import os
import json
import sqlite3
import time
from flask import Flask, request, jsonify

app = Flask(__name__)
DB_PATH = os.environ.get("DB_PATH", "catalogue.db")

def get_conn():
    conn = sqlite3.connect(DB_PATH)
    conn.row_factory = sqlite3.Row
    return conn

@app.route("/products", methods=["GET"])
def list_products():
    conn = get_conn()
    rows = conn.execute("SELECT * FROM products").fetchall()
    conn.close()
    return jsonify([dict(r) for r in rows])

@app.route("/products/<int:pid>", methods=["GET"])
def get_product(pid):
    conn = get_conn()
    row = conn.execute("SELECT * FROM products WHERE id=?", [pid]).fetchone()
    conn.close()
    if row is None:
        return jsonify({"error": "not found"}), 404
    return jsonify(dict(row))

@app.route("/products", methods=["POST"])
def create_product():
    body = request.get_json()
    ts = int(time.time())
    conn = get_conn()
    conn.execute(
        "INSERT INTO products (name, price, created_at) VALUES (?,?,?)",
        [body["name"], body["price"], ts]
    )
    conn.commit()
    conn.close()
    return jsonify({"status": "created", "at": ts}), 201

if __name__ == "__main__":
    port = int(os.environ.get("PORT", 8080))
    app.run(port=port)
```

### UniLang Migration (~25 lines)

```python
// product_catalogue.uniL
// All stdlib imports removed — builtins handle everything

DB = env_get("DB_PATH") or "catalogue.db"
conn = db_connect(DB)

def list_products(req):
    rows = db_query(conn, "SELECT * FROM products", [])
    return rows

def get_product(req):
    pid = int(req.params["pid"])
    rows = db_query(conn, "SELECT * FROM products WHERE id=?", [pid])
    if len(rows) == 0:
        return {"error": "not found"}, 404
    return rows[0]

def create_product(req):
    body = req.body
    ts = int(now())
    db_query(
        conn,
        "INSERT INTO products (name, price, created_at) VALUES (?,?,?)",
        [body["name"], body["price"], ts]
    )
    db_commit(conn)
    return {"status": "created", "at": ts}

serve(
    int(env_get("PORT") or 8080),
    {
        "GET /products":         list_products,
        "GET /products/:pid":    get_product,
        "POST /products":        create_product
    }
)
```

### What Changed

| Aspect | Python/Flask | UniLang |
|---|---|---|
| Imports | 6 import lines | 0 |
| App factory | `Flask(__name__)` boilerplate | removed |
| DB connection | `sqlite3.connect()` + `row_factory` per route | single `db_connect()` at top |
| JSON serialisation | `jsonify(...)` wrapper | return dict directly |
| Timestamp | `int(time.time())` | `int(now())` |
| Env vars | `os.environ.get(k, default)` | `env_get(k) or default` |
| Server startup | `app.run(port=port)` + `if __name__ == "__main__"` | `serve(port, routes)` |
| Total lines | ~35 | ~25 |

---

## Next Steps

- **Language Spec** — `/docs/specifications/LANGUAGE_SPEC.md` — full grammar, type system, and interop details.
- **Quickstart** — `/docs/guides/QUICKSTART.md` — run your first `.uniL` file in 5 minutes.
- **Builtin Reference** — `/docs/specifications/BUILTINS.md` — complete list of every builtin function with signatures and return types.
- **Java Interop** — Language Spec section 13 — how to call Java libraries from UniLang using the `bridge` keyword.
