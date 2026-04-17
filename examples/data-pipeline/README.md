# Data Processing / ETL Pipeline — UniLang Example

A self-contained **Extract → Transform → Load → Report** pipeline written in pure UniLang.  
It demonstrates CSV parsing, data validation, revenue computation, SQLite persistence, and aggregate queries — all in one file, no external dependencies.

---

## Files

| File | Purpose |
|---|---|
| `pipeline.uniL` | The entire ETL pipeline — run this |

---

## Pipeline Stages

### 1. Extract
The CSV data is embedded directly in the script as a multi-line string (`RAW_CSV`).  
`parse_csv()` splits on newlines to get rows, then splits each row on commas and zips the values against the header names to produce a list of dicts:

```
[
  {"date": "2024-01-05", "product": "Laptop Pro 15", "quantity": "3", ...},
  ...
]
```

Schema: `date | product | category | region | quantity | unit_price`

### 2. Transform
`transform_rows()` applies three transformations in a single pass:

- **Filter** — drops rows where `quantity <= 0` or any required field is empty (two intentionally bad rows are included in the dataset).
- **Cast** — converts `quantity` to `int` and `unit_price` to `float`.
- **Enrich** — computes `revenue = quantity * unit_price` and extracts a `month` string (`YYYY-MM`) from the date for later aggregation.

Two helper aggregations are then built in memory:

- `group_by_product()` — total quantity, total revenue, order count, average order revenue per product.
- `group_by_month()` — total revenue and units sold per calendar month.

### 3. Load
`init_db()` opens (or creates) `sales_pipeline.db` and ensures three tables exist:

| Table | Contents |
|---|---|
| `sales` | One row per cleaned transaction |
| `product_summary` | Pre-aggregated totals per product |
| `monthly_summary` | Pre-aggregated totals per month |

The tables are cleared at the start of each run so the script is **idempotent** — safe to run multiple times.

`db_connect`, `db_exec`, and `db_query` are UniLang built-in SQLite bindings.

### 4. Query & Report
Five queries are run against the loaded tables to produce the final report:

- Overall totals (transaction count, units sold, grand revenue)
- Top 3 products by revenue
- Monthly revenue trend (ordered by month)
- Revenue and unit breakdown by category
- Revenue and order count breakdown by region

Results are printed to stdout as a formatted table.

---

## How to Run

From the repository root:

```bash
unilang run examples/data-pipeline/pipeline.uniL
```

The script writes `sales_pipeline.db` in the current working directory.

---

## Expected Output

```
Starting ETL pipeline...

[1/4] Extract — parsing inline CSV...
      34 raw rows read.
[2/4] Transform — filtering, casting, computing revenue...
      32 valid rows, 2 skipped.
      Aggregated 7 products across 3 months.
[3/4] Load — writing to SQLite (sales_pipeline.db)...
      32 sale rows inserted.
      7 product summary rows inserted.
      3 monthly summary rows inserted.
[4/4] Report — querying aggregates and printing summary...

╔══════════════════════════════════════════════════════════╗
║          SALES ETL PIPELINE — SUMMARY REPORT            ║
╚══════════════════════════════════════════════════════════╝

── OVERVIEW ──────────────────────────────────────────────
  Rows parsed from CSV    : 34
  Rows skipped (invalid)  : 2
  Rows loaded to SQLite   : 32
  Total units sold        : 302
  Grand total revenue     : $XX,XXX.XX

── TOP 3 PRODUCTS BY REVENUE ─────────────────────────────
  Product                Revenue       Units   Orders
  -------                -------       -----   ------
  1. Laptop Pro 15       $XXXXX.XX     20      9
  2. Standing Desk       $XXXXX.XX     10      4
  3. Ergonomic Keyboard  $XXXXX.XX     40      4

── MONTHLY REVENUE TREND ──────────────────────────────────
  Month      Revenue       Units Sold
  -----      -------       ----------
  2024-01    $X,XXX.XX     ...
  2024-02    $X,XXX.XX     ...
  2024-03    $X,XXX.XX     ...

── REVENUE BY CATEGORY ────────────────────────────────────
  Electronics   $XX,XXX.XX  (XXX units)
  Furniture     $X,XXX.XX   (XX units)

── REVENUE BY REGION ──────────────────────────────────────
  North    $X,XXX.XX  (XX orders)
  ...

Pipeline complete. Database: sales_pipeline.db
```

---

## Key UniLang Patterns Shown

- **String splitting** for CSV parsing (`text.split("\n")`, `line.split(",")`)
- **Type casting** with `int()` and `float()`
- **Parallel arrays** as a dict-group workaround (same pattern used in `server.uniL` dashboard)
- **SQLite built-ins**: `db_connect`, `db_exec` (DDL + DML), `db_query` (SELECT → list of dicts)
- **Parameterised queries** with `?` placeholders to prevent injection
- **Idempotent loads** via `DELETE FROM` before each insert batch
