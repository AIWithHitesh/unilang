# Java → UniLang Migration Guide

**Version:** 1.0.0  
**Applies to:** UniLang 1.0.0-draft

---

## Table of Contents

1. [Introduction](#1-introduction)
2. [Side-by-Side Syntax Table](#2-side-by-side-syntax-table)
3. [What You Can Keep](#3-what-you-can-keep)
4. [What to Change](#4-what-to-change)
5. [Mapping Java Stdlib to UniLang Builtins](#5-mapping-java-stdlib-to-unilang-builtins)
6. [Common Migration Patterns](#6-common-migration-patterns)
7. [Complete Migration Example](#7-complete-migration-example)

---

## 1. Introduction

UniLang is a unified language that treats Java syntax and Python syntax as two equally valid dialects of the same language. A `.uniL` file can contain Java-style classes, Python-style functions, and top-level statements all side by side — they compile to the same Unified IR and interoperate without any bridging code.

### Why migrate from Java to UniLang?

- **Zero rewrite cost for Java code.** Your existing Java class declarations, type annotations, generics, access modifiers, and control flow work unchanged inside a `.uniL` file. Migration can be incremental — copy a class in, and it runs.
- **Eliminate boilerplate.** No `public static void main`, no `import java.util.*`, no wrapper libraries for HTTP or databases. Common infrastructure is built in.
- **Mix paradigms freely.** Add a Python-style list comprehension inside a Java method, or call a `def` function from a Java class, without any FFI ceremony.
- **Single runtime, unified types.** `String`, `List<T>`, `Dict<K,V>`, and `Optional<T>` are the same types regardless of which syntax created them.

### What stays the same

Java syntax is natively supported — not translated or shimmed. The following work exactly as they do in Java:

- Class declarations with access modifiers (`public`, `private`, `protected`)
- Constructors, instance methods, static methods
- `extends`, `implements`, `interface`, `abstract`, `final`
- `new`, `this`, `super`
- Generics (`List<T>`, `Map<K,V>`, bounded `<T extends Number>`)
- `try` / `catch` / `finally`
- `for` loops (both C-style and enhanced)
- `instanceof`, `switch` expressions (Java 21 style)
- Java-style lambda syntax (`(a, b) -> a + b`)
- Javadoc comments (`/** ... */`)
- Semicolons (optional but accepted everywhere)

### What changes

| Concern | Java | UniLang |
|---------|------|---------|
| Entry point | `public static void main(String[] args)` | Top-level code runs directly |
| Standard output | `System.out.println(...)` | `print(...)` (also `System.out.println` still works) |
| Imports | `import java.util.ArrayList` | No java.* imports — collections are builtins |
| HTTP server | Spring Boot / JAX-RS library | `serve(port, router)` builtin |
| Database | JDBC + driver JAR | `db_connect / db_query / db_exec` builtins |
| JSON | Jackson / Gson library | `to_json / from_json` builtins |
| Environment vars | `System.getenv("KEY")` | `env_get("KEY")` |

---

## 2. Side-by-Side Syntax Table

### Class declaration

```java
// Java
public class Product {
    private String id;
    private double price;
}
```

```unilang
// UniLang — identical, works as-is
public class Product {
    private String id;
    private double price;
}
```

### Constructor

```java
// Java
public class Product {
    private String id;
    private double price;

    public Product(String id, double price) {
        this.id = id;
        this.price = price;
    }
}
```

```unilang
// UniLang — identical
public class Product {
    private String id;
    private double price;

    public Product(String id, double price) {
        this.id = id;
        this.price = price;
    }
}
```

### Instance methods

```java
// Java
public String getId() {
    return this.id;
}

public void applyDiscount(double pct) {
    this.price = this.price * (1.0 - pct);
}
```

```unilang
// UniLang — identical
public String getId() {
    return this.id;
}

public void applyDiscount(double pct) {
    this.price = this.price * (1.0 - pct);
}
```

### Static methods

```java
// Java
public static Product fromMap(Map<String, Object> data) {
    return new Product((String) data.get("id"), (double) data.get("price"));
}
```

```unilang
// UniLang — identical; Dict<String, Dynamic> is the idiomatic type
public static Product fromMap(Dict<String, Dynamic> data) {
    return new Product((String) data["id"], (double) data["price"]);
}
```

### Inheritance

```java
// Java
public class DigitalProduct extends Product {
    private String downloadUrl;

    public DigitalProduct(String id, double price, String url) {
        super(id, price);
        this.downloadUrl = url;
    }

    @Override
    public String toString() {
        return "Digital: " + super.getId();
    }
}
```

```unilang
// UniLang — identical
public class DigitalProduct extends Product {
    private String downloadUrl;

    public DigitalProduct(String id, double price, String url) {
        super(id, price);
        this.downloadUrl = url;
    }

    @Override
    public String toString() {
        return "Digital: " + super.getId();
    }
}
```

### try / catch / finally

```java
// Java
try {
    int result = Integer.parseInt(input);
    process(result);
} catch (NumberFormatException e) {
    System.out.println("Bad input: " + e.getMessage());
} finally {
    cleanup();
}
```

```unilang
// UniLang — identical (Java-style catch syntax)
try {
    int result = Integer.parseInt(input);
    process(result);
} catch (NumberFormatException e) {
    print("Bad input: " + e.getMessage());
} finally {
    cleanup();
}

// Also accepted: Python-style except
try {
    result = int(input)
    process(result)
} except ValueError as e:
    print(f"Bad input: {e}")
```

### for loop — both styles

```java
// Java C-style for
for (int i = 0; i < 10; i++) {
    System.out.println(i);
}

// Java enhanced for
for (String item : items) {
    System.out.println(item);
}
```

```unilang
// UniLang — both work unchanged
for (int i = 0; i < 10; i++) {
    print(i);
}

for (String item : items) {
    print(item);
}

// Also accepted: Python-style
for i in range(10):
    print(i)

for item in items:
    print(item)
```

### Generics

```java
// Java
List<String> names = new ArrayList<>();
Map<String, Integer> counts = new HashMap<>();

public <T extends Comparable<T>> T max(List<T> list) {
    T best = list.get(0);
    for (T x : list) {
        if (x.compareTo(best) > 0) best = x;
    }
    return best;
}
```

```unilang
// UniLang — identical generics syntax
List<String> names = new ArrayList<>()
Dict<String, Integer> counts = {}   // Dict is the idiomatic map type

public <T extends Comparable<T>> T max(List<T> list) {
    T best = list[0]
    for (T x : list) {
        if (x.compareTo(best) > 0) best = x
    }
    return best
}
```

### null checks

```java
// Java
if (user != null) {
    System.out.println(user.getName());
}

// Java null coalescing (ternary)
String name = (user != null) ? user.getName() : "Guest";

// Java Optional
Optional.ofNullable(user).ifPresent(u -> process(u));
```

```unilang
// UniLang — null and None are aliases; both work
if (user != null) {
    print(user.getName());
}

// UniLang null coalescing operator
String name = user?.getName() ?? "Guest"

// Python-style None check also valid
if user is not None:
    print(user.getName())
```

### String operations

```java
// Java
String s = "Hello, World!";
int len = s.length();
String upper = s.toUpperCase();
String sub = s.substring(0, 5);
boolean starts = s.startsWith("Hello");
String[] parts = s.split(", ");
String trimmed = s.trim();
String formatted = String.format("Hi %s, you are %d", name, age);
```

```unilang
// UniLang — Java string methods work; Python builtins also available
String s = "Hello, World!"
int len = len(s)                  // or s.length()
String upper = s.upper()          // or s.toUpperCase()
String sub = s[0:5]               // or s.substring(0, 5)
bool starts = s.startsWith("Hello")
List<String> parts = s.split(", ")
String trimmed = s.strip()        // or s.trim()
String formatted = f"Hi {name}, you are {age}"  // or String.format(...)
```

---

## 3. What You Can Keep

The following Java constructs are accepted in UniLang **without any modification**. Copy them from a `.java` file into a `.uniL` file and they will compile.

| Construct | Example |
|-----------|---------|
| Braces for blocks | `public void foo() { ... }` |
| Semicolons | `int x = 5;` (optional everywhere) |
| Primitive type annotations | `int`, `double`, `boolean`, `char`, `long` |
| Reference type annotations | `String`, `Integer`, `List<T>`, `Map<K,V>` |
| `new` keyword | `new ArrayList<>()`, `new Product("p1", 9.99)` |
| `this` and `super` | `this.name = name; super.init()` |
| Access modifiers | `public`, `private`, `protected` |
| `static`, `final`, `abstract` | `public static final int MAX = 100` |
| `@Override`, `@Deprecated` | Standard Java annotations |
| `implements`, `interface` | `public interface Serializable { ... }` |
| `extends` | `public class Dog extends Animal` |
| `enum` | `public enum Status { ACTIVE, INACTIVE }` |
| `instanceof` | `if (obj instanceof String s) { ... }` |
| Switch expressions | `String r = switch(x) { case 1 -> "a"; default -> "b"; }` |
| Java lambdas | `list.sort((a, b) -> a.compareTo(b))` |
| Ternary operator | `int max = a > b ? a : b` |
| Java-style casts | `(String) obj`, `(int) value` |
| `throw` / `throws` | `throw new IllegalArgumentException("bad")` |
| Varargs | `public void log(String... msgs)` |
| Javadoc | `/** @param name the user's name */` |

---

## 4. What to Change

### 4.1 Remove `import java.*`

Java standard library imports are not available because their functionality is provided by UniLang builtins or the unified type system.

```java
// Java — remove these
import java.util.ArrayList;
import java.util.HashMap;
import java.util.List;
import java.util.Map;
import java.util.Optional;
import java.io.IOException;
```

```unilang
// UniLang — no imports needed; these are builtins
List<String> items = []           // built-in list
Dict<String, Integer> map = {}    // built-in dict
Optional<String> opt = None       // Optional<T> is a built-in type
```

Third-party imports (e.g., `import com.mycompany.util.Helper`) should be replaced with a UniLang module import:

```unilang
import mymodule.Helper
```

### 4.2 Replace `System.out.println` with `print`

`System.out.println` continues to work for compatibility, but `print` is the idiomatic UniLang form and supports f-strings.

```java
// Java
System.out.println("Processing item: " + item.getId());
System.out.printf("Price: %.2f%n", item.getPrice());
```

```unilang
// UniLang
print(f"Processing item: {item.getId()}")
print(f"Price: {item.getPrice():.2f}")
```

### 4.3 Remove `public static void main`

UniLang files execute top-level statements directly. There is no required entry-point method.

```java
// Java — remove this wrapper
public class App {
    public static void main(String[] args) {
        Server server = new Server();
        server.start(8080);
    }
}
```

```unilang
// UniLang — top-level code runs directly
server = new Server()
server.start(8080)

// Or mix with class definition in the same file:
public class Server {
    public void start(int port) { serve(port, router) }
}
server = new Server()
server.start(8080)
```

### 4.4 Replace library-based HTTP, DB, and JSON with builtins

| Java approach | UniLang builtin |
|---------------|-----------------|
| Spring Boot `@RestController` | `serve(port, router)` |
| JDBC `DriverManager.getConnection(url)` | `db_connect(url)` |
| JDBC `stmt.executeQuery(sql)` | `db_query(sql, params)` |
| JDBC `stmt.executeUpdate(sql)` | `db_exec(sql, params)` |
| Jackson `objectMapper.writeValueAsString(obj)` | `to_json(obj)` |
| Jackson `objectMapper.readValue(json, Map.class)` | `from_json(json)` |
| `System.getenv("KEY")` | `env_get("KEY")` |

See sections 5 and 6 for detailed usage.

---

## 5. Mapping Java Stdlib to UniLang Builtins

### Math

| Java | UniLang |
|------|---------|
| `Math.abs(x)` | `abs(x)` |
| `Math.max(a, b)` | `max(a, b)` |
| `Math.min(a, b)` | `min(a, b)` |
| `Math.pow(base, exp)` | `base ** exp` or `pow(base, exp)` |
| `Math.sqrt(x)` | `x ** 0.5` |
| `Math.floor(x)` | `int(x)` or `x // 1` |
| `Math.round(x)` | `round(x)` |
| `Math.random()` | `random()` |

### String

| Java | UniLang |
|------|---------|
| `s.length()` | `len(s)` or `s.length()` |
| `s.toUpperCase()` | `s.upper()` |
| `s.toLowerCase()` | `s.lower()` |
| `s.trim()` | `s.strip()` |
| `s.substring(a, b)` | `s[a:b]` |
| `s.contains(sub)` | `sub in s` |
| `s.startsWith(p)` | `s.startsWith(p)` |
| `s.endsWith(p)` | `s.endsWith(p)` |
| `s.replace(a, b)` | `s.replace(a, b)` |
| `s.split(delim)` | `s.split(delim)` |
| `s.isEmpty()` | `len(s) == 0` or `not s` |
| `String.valueOf(x)` | `str(x)` |
| `String.join(sep, list)` | `sep.join(list)` |
| `String.format("Hi %s", name)` | `f"Hi {name}"` |
| `Integer.parseInt(s)` | `int(s)` |
| `Double.parseDouble(s)` | `float(s)` |

### Collections

| Java | UniLang |
|------|---------|
| `new ArrayList<>()` | `[]` |
| `list.add(x)` | `list.append(x)` |
| `list.get(i)` | `list[i]` |
| `list.set(i, x)` | `list[i] = x` |
| `list.remove(i)` | `list.pop(i)` or `del list[i]` |
| `list.size()` | `len(list)` |
| `list.isEmpty()` | `len(list) == 0` or `not list` |
| `list.contains(x)` | `x in list` |
| `Collections.sort(list)` | `list.sort()` |
| `new HashMap<>()` | `{}` |
| `map.put(k, v)` | `map[k] = v` |
| `map.get(k)` | `map[k]` |
| `map.getOrDefault(k, d)` | `map.get(k, d)` |
| `map.containsKey(k)` | `k in map` |
| `map.remove(k)` | `del map[k]` |
| `map.size()` | `len(map)` |
| `map.keySet()` | `map.keys()` |
| `map.values()` | `map.values()` |
| `map.entrySet()` | `map.items()` |
| `new HashSet<>()` | `set()` or `{x, y, z}` |
| `set.add(x)` | `set.add(x)` |
| `set.contains(x)` | `x in set` |

### System / Environment

| Java | UniLang |
|------|---------|
| `System.getenv("KEY")` | `env_get("KEY")` |
| `System.currentTimeMillis()` | `time()` |
| `Thread.sleep(ms)` | `sleep(ms / 1000.0)` |
| `UUID.randomUUID().toString()` | `uuid()` |

---

## 6. Common Migration Patterns

### 6.1 Spring Boot REST controller → `serve()`

```java
// Java — Spring Boot
@RestController
@RequestMapping("/products")
public class ProductController {

    @Autowired
    private ProductService service;

    @GetMapping("/{id}")
    public ResponseEntity<Product> getById(@PathVariable String id) {
        Product p = service.findById(id);
        if (p == null) return ResponseEntity.notFound().build();
        return ResponseEntity.ok(p);
    }

    @PostMapping
    public ResponseEntity<Product> create(@RequestBody Product product) {
        Product saved = service.save(product);
        return ResponseEntity.status(201).body(saved);
    }
}
```

```unilang
// UniLang
router = {}

router["GET /products/:id"] = def(req) {
    id = req["params"]["id"]
    rows = db_query("SELECT * FROM products WHERE id = ?", [id])
    if len(rows) == 0 {
        return {"status": 404, "body": to_json({"error": "not found"}),
                "content_type": "application/json"}
    }
    return {"status": 200, "body": to_json(rows[0]),
            "content_type": "application/json"}
}

router["POST /products"] = def(req) {
    product = from_json(req["body"])
    db_exec("INSERT INTO products (id, name, price) VALUES (?, ?, ?)",
            [product["id"], product["name"], product["price"]])
    return {"status": 201, "body": to_json(product),
            "content_type": "application/json"}
}

serve(8080, router)
```

### 6.2 JDBC → `db_connect / db_query / db_exec`

```java
// Java — JDBC
Connection conn = DriverManager.getConnection(
    "jdbc:sqlite:myapp.db", "", "");

PreparedStatement ps = conn.prepareStatement(
    "SELECT * FROM users WHERE email = ?");
ps.setString(1, email);
ResultSet rs = ps.executeQuery();

while (rs.next()) {
    String name = rs.getString("name");
    System.out.println(name);
}
conn.close();
```

```unilang
// UniLang
db_connect("sqlite://myapp.db")

rows = db_query("SELECT * FROM users WHERE email = ?", [email])
for row in rows:
    print(row["name"])

// Writes use db_exec
db_exec("INSERT INTO users (id, email, name) VALUES (?, ?, ?)",
        [uuid(), email, name])
```

### 6.3 Jackson JSON → `to_json / from_json`

```java
// Java — Jackson
ObjectMapper mapper = new ObjectMapper();

// Serialize
String json = mapper.writeValueAsString(product);

// Deserialize
Map<String, Object> data = mapper.readValue(jsonString, Map.class);
String name = (String) data.get("name");
```

```unilang
// UniLang
// Serialize
String json = to_json(product)

// Deserialize — returns a Dict
data = from_json(json_string)
String name = data["name"]
```

### 6.4 Java Optional → UniLang null coalescing

```java
// Java
Optional<User> optUser = userRepo.findById(id);
String name = optUser.map(User::getName).orElse("Anonymous");
```

```unilang
// UniLang
user = find_user_by_id(id)          // returns null if not found
String name = user?.getName() ?? "Anonymous"
```

### 6.5 Java streams → UniLang list comprehensions

```java
// Java streams
List<String> names = users.stream()
    .filter(u -> u.isActive())
    .map(User::getName)
    .sorted()
    .collect(Collectors.toList());
```

```unilang
// UniLang — list comprehension
List<String> names = sorted([u.getName() for u in users if u.isActive()])
```

---

## 7. Complete Migration Example

### Before — Java CRUD service (Spring Boot + JDBC)

```java
// ProductService.java
package com.example;

import java.sql.*;
import java.util.*;
import org.springframework.web.bind.annotation.*;
import org.springframework.http.ResponseEntity;
import com.fasterxml.jackson.databind.ObjectMapper;

@RestController
@RequestMapping("/api/products")
public class ProductService {

    private static Connection conn;
    private static ObjectMapper mapper = new ObjectMapper();

    static {
        try {
            conn = DriverManager.getConnection("jdbc:sqlite:store.db");
            conn.createStatement().execute(
                "CREATE TABLE IF NOT EXISTS products " +
                "(id TEXT PRIMARY KEY, name TEXT, price REAL, stock INT)");
        } catch (SQLException e) {
            throw new RuntimeException(e);
        }
    }

    @GetMapping
    public ResponseEntity<List<Map<String,Object>>> listAll() throws Exception {
        ResultSet rs = conn.createStatement()
            .executeQuery("SELECT * FROM products");
        List<Map<String,Object>> results = new ArrayList<>();
        while (rs.next()) {
            Map<String,Object> row = new HashMap<>();
            row.put("id",    rs.getString("id"));
            row.put("name",  rs.getString("name"));
            row.put("price", rs.getDouble("price"));
            row.put("stock", rs.getInt("stock"));
            results.add(row);
        }
        return ResponseEntity.ok(results);
    }

    @GetMapping("/{id}")
    public ResponseEntity<?> getById(@PathVariable String id) throws Exception {
        PreparedStatement ps = conn.prepareStatement(
            "SELECT * FROM products WHERE id = ?");
        ps.setString(1, id);
        ResultSet rs = ps.executeQuery();
        if (!rs.next()) return ResponseEntity.notFound().build();
        Map<String,Object> row = new HashMap<>();
        row.put("id",    rs.getString("id"));
        row.put("name",  rs.getString("name"));
        row.put("price", rs.getDouble("price"));
        row.put("stock", rs.getInt("stock"));
        return ResponseEntity.ok(row);
    }

    @PostMapping
    public ResponseEntity<?> create(@RequestBody String body) throws Exception {
        Map data = mapper.readValue(body, Map.class);
        String id = UUID.randomUUID().toString();
        PreparedStatement ps = conn.prepareStatement(
            "INSERT INTO products (id, name, price, stock) VALUES (?,?,?,?)");
        ps.setString(1, id);
        ps.setString(2, (String) data.get("name"));
        ps.setDouble(3, ((Number) data.get("price")).doubleValue());
        ps.setInt(4, ((Number) data.get("stock")).intValue());
        ps.executeUpdate();
        data.put("id", id);
        return ResponseEntity.status(201).body(data);
    }

    @DeleteMapping("/{id}")
    public ResponseEntity<?> delete(@PathVariable String id) throws Exception {
        PreparedStatement ps = conn.prepareStatement(
            "DELETE FROM products WHERE id = ?");
        ps.setString(1, id);
        int affected = ps.executeUpdate();
        if (affected == 0) return ResponseEntity.notFound().build();
        return ResponseEntity.noContent().build();
    }

    public static void main(String[] args) {
        SpringApplication.run(ProductService.class, args);
    }
}
```

### After — UniLang equivalent

```unilang
// products.uniL  —  run with: unilang run products.uniL

// ── Database setup ────────────────────────────────────────────
db_connect("sqlite://store.db")
db_exec("CREATE TABLE IF NOT EXISTS products (id TEXT PRIMARY KEY, name TEXT, price REAL, stock INT)", [])

// ── Helper ────────────────────────────────────────────────────
def ok(data, status=200) {
    return {"status": status, "body": to_json(data), "content_type": "application/json"}
}
def err(msg, status=404) {
    return {"status": status, "body": to_json({"error": msg}), "content_type": "application/json"}
}

// ── Handlers ──────────────────────────────────────────────────
router = {}

router["GET /api/products"] = def(req) {
    rows = db_query("SELECT * FROM products", [])
    return ok(rows)
}

router["GET /api/products/:id"] = def(req) {
    rows = db_query("SELECT * FROM products WHERE id = ?", [req["params"]["id"]])
    if len(rows) == 0 { return err("not found") }
    return ok(rows[0])
}

router["POST /api/products"] = def(req) {
    data = from_json(req["body"])
    id = uuid()
    db_exec("INSERT INTO products (id, name, price, stock) VALUES (?, ?, ?, ?)",
            [id, data["name"], data["price"], data["stock"]])
    data["id"] = id
    return ok(data, 201)
}

router["DELETE /api/products/:id"] = def(req) {
    db_exec("DELETE FROM products WHERE id = ?", [req["params"]["id"]])
    return {"status": 204, "body": "", "content_type": "application/json"}
}

// ── Start server ──────────────────────────────────────────────
print("Listening on http://localhost:8080")
serve(8080, router)
```

The UniLang version is roughly one-third the line count. The logic is identical; the boilerplate (imports, static initializer, checked exceptions, response-entity wrappers, manual result-set mapping) is gone because it is handled by the runtime.

---

## Summary

| Step | Action |
|------|--------|
| 1 | Rename `.java` to `.uniL` — class, method, and type syntax compile unchanged |
| 2 | Remove all `import java.*` statements |
| 3 | Delete the `public static void main` wrapper; promote its body to top-level |
| 4 | Replace `System.out.println` with `print` |
| 5 | Replace JDBC setup with `db_connect` + `db_query` / `db_exec` |
| 6 | Replace Spring/JAX-RS controllers with `serve(port, router)` |
| 7 | Replace Jackson/Gson with `to_json` / `from_json` |
| 8 | Replace `System.getenv` with `env_get` |
| 9 | Optionally simplify collections and string ops using Python-style builtins |
