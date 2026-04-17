# Task Manager API — UniLang Web Service Example

A complete REST API for a **to-do / task management** service, written in pure UniLang.  
Demonstrates the core web-service capabilities of the UniLang runtime:

- `serve(port, router)` — built-in HTTP server
- `db_connect` / `db_exec` / `db_query` — embedded SQLite
- `to_json` / `from_json` — JSON serialisation helpers
- `try / except` — error handling
- Python-style functions and control flow alongside Java-style structured code

---

## How to Run

From the **repository root**:

```bash
unilang run examples/web-service/server.uniL
```

The server starts on **http://localhost:8080** and creates `tasks.db` (SQLite) in the working directory on first run.

---

## Data Model

### Task

| Field         | Type     | Notes                                           |
|---------------|----------|-------------------------------------------------|
| `id`          | integer  | Auto-assigned by SQLite                         |
| `title`       | string   | **Required**                                    |
| `description` | string   | Optional, defaults to `""`                      |
| `status`      | string   | `pending` · `in_progress` · `done` · `cancelled` |
| `priority`    | string   | `low` · `medium` · `high`                       |
| `due_date`    | string   | Free-form date string, e.g. `"2026-05-01"`      |
| `tags`        | string[] | Stored in a separate `tags` table               |
| `created_at`  | string   | Set automatically on creation                   |
| `updated_at`  | string   | Updated automatically on every PUT              |

---

## Endpoints

| Method   | Path          | Description                             |
|----------|---------------|-----------------------------------------|
| `GET`    | `/`           | Service info and endpoint listing       |
| `GET`    | `/health`     | Liveness check with task count summary  |
| `GET`    | `/tasks`      | List all tasks (filterable)             |
| `POST`   | `/tasks`      | Create a new task                       |
| `GET`    | `/tasks/:id`  | Get a single task by ID                 |
| `PUT`    | `/tasks/:id`  | Update a task (partial update)          |
| `DELETE` | `/tasks/:id`  | Delete a task                           |

### Query Parameters for `GET /tasks`

| Parameter  | Example              | Description                        |
|------------|----------------------|------------------------------------|
| `status`   | `?status=pending`    | Filter by status                   |
| `priority` | `?priority=high`     | Filter by priority                 |
| `search`   | `?search=docs`       | Search title and description       |

---

## Example curl Commands

### Welcome / index

```bash
curl http://localhost:8080/
```

### Health check

```bash
curl http://localhost:8080/health
```

### Create a task (minimal)

```bash
curl -X POST http://localhost:8080/tasks \
     -H "Content-Type: application/json" \
     -d '{"title": "Buy groceries"}'
```

### Create a task (all fields)

```bash
curl -X POST http://localhost:8080/tasks \
     -H "Content-Type: application/json" \
     -d '{
           "title":       "Write unit tests",
           "description": "Cover all handler functions with edge cases",
           "priority":    "high",
           "due_date":    "2026-04-30",
           "tags":        ["testing", "backend"]
         }'
```

### List all tasks

```bash
curl http://localhost:8080/tasks
```

### List tasks filtered by status

```bash
curl "http://localhost:8080/tasks?status=pending"
```

### List tasks filtered by priority

```bash
curl "http://localhost:8080/tasks?priority=high"
```

### Search tasks

```bash
curl "http://localhost:8080/tasks?search=test"
```

### Get a single task

```bash
curl http://localhost:8080/tasks/1
```

### Update a task (partial — only supply the fields to change)

```bash
curl -X PUT http://localhost:8080/tasks/1 \
     -H "Content-Type: application/json" \
     -d '{"status": "in_progress"}'
```

### Mark a task done and update its tags

```bash
curl -X PUT http://localhost:8080/tasks/1 \
     -H "Content-Type: application/json" \
     -d '{
           "status": "done",
           "tags":   ["testing", "backend", "completed"]
         }'
```

### Delete a task

```bash
curl -X DELETE http://localhost:8080/tasks/1
```

---

## HTTP Status Codes Used

| Code | Meaning                                  |
|------|------------------------------------------|
| 200  | OK — successful GET / PUT / DELETE       |
| 201  | Created — successful POST                |
| 204  | No Content — OPTIONS preflight           |
| 400  | Bad Request — validation error           |
| 404  | Not Found — task ID does not exist       |
| 405  | Method Not Allowed                       |
| 500  | Internal Server Error — unexpected fault |
| 503  | Service Unavailable — DB unreachable     |

---

## Project Structure

```
examples/web-service/
├── server.uniL   # Full REST API — single-file UniLang server
└── README.md     # This file
```

The database file `tasks.db` is created automatically in the working directory when the server first starts.
