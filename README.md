# WhichOne API

## Description
WhichOne API is a high-performance, concurrent, and lightweight polling backend application built using the Rust programming language. The primary function of this service is to manage digital polls, tags, and vote calculations in real-time. It provides robust RESTful endpoints that allow client applications to create public polls, retrieve lists of active polls, search and filter specific topics, update existing constraints, and securely cast or track votes. 

Designed with memory safety and thread concurrency at its core, the application manages data state in-memory via safe concurrent patterns, eliminating traditional database overhead during high-throughput voting scenarios.

---

## Prerequisites
To compile and execute this project locally, ensure you have the following requirements installed:

* **Rust Compiler & Cargo:** Version 1.95.0 or higher.
* **Rust Edition:** 2024.

### Dependencies
The project relies on the following core crates:
* `axum` (v0.8.9) - Ergonomic and asynchronous web framework.
* `tokio` (v1.52.3 with `full` features) - Asynchronous runtime.
* `serde` (v1.0.228 with `derive` features) - Serialization and deserialization framework.
* `uuid` (v1.23.3 with `v4` and `serde` features) - Universally Unique Identifier generation.

### Environment Setup
Create a `.env` file in the root directory of the project to configure the network socket variables:

```env
SERVER_ADDR=127.0.0.1
SERVER_PORT=8080
```

---

## Getting Started

Follow these steps to clone the repository, compile the source code, and run the server locally.

### 1. Clone the Repository

```bash
git clone https://github.com/naufalhanif25/whichone-api.git
cd whichone-api
```

### 2. Build the Project

To compile the dependencies and build the binary in development mode:

```bash
cargo build
```

### 3. Run the Application

Start the Axum server locally using Cargo:

```bash
cargo run
```

Once initialized, the terminal will display: `Server is running at http://127.0.0.1:8080`. To safely terminate the process, press `Ctrl + C`. The server will execute a graceful shutdown sequence.

---

## API Endpoints Documentation

All response bodies are wrapped in a unified structure to enforce consistency across standard operations and error handlers:

```json
{
    "code": "unsigned 16-bit integer representing HTTP status",
    "messages": "string description of the result or error context",
    "data": "generic JSON object/array or null"
}
```

### 1. Poll Management

#### Create a New Poll

* **URL:** `/polls`
* **Method:** `POST`
* **Request Body:**
```json
{
    "title": "Favorite Backend Framework",
    "tags": ["programming", "backend"],
    "options": [
        {
            "option": "Axum (Rust)",
            "reference": "https://github.com/tokio-rs/axum"
        },
        {
            "option": "Express (JavaScript)",
            "reference": null
        }
    ]
}
```

* **Responses:**
* **201 Created:**
```json
{
    "code": 201,
    "messages": "Poll created successfully",
    "data": {
        "id": "550e8400-e29b-41d4-a716-446655440000",
        "title": "Favorite Backend Framework",
        "tags": ["programming", "backend"],
        "options": [
            { 
                "opt_id": "a1b2c3d4-e5f6-7a8b-9c0d-1e2f3a4b5c6d", 
                "option": "Axum (Rust)", 
                "reference": "https://github.com/tokio-rs/axum", 
                "vote": 0 
            },
            { 
                "opt_id": "f6e5d4c3-b2a1-0f9e-8d7c-6b5a4f3e2d1c", 
                "option": "Express (JavaScript)", 
                "reference": null, 
                "vote": 0 
            }
        ]
    }
}
```


* **400 Bad Request (Empty Payload fields):**
```json
{
    "code": 400,
    "messages": "Payload cannot be empty",
    "data": null
}
```

#### Get All Polls

* **URL:** `/polls`
* **Method:** `GET`
* **Response (200 OK):**
```json
{
    "code": 200,
    "messages": "Polls retrieved successfully",
    "data": [
        {
            "id": "550e8400-e29b-41d4-a716-446655440000",
            "title": "Favorite Backend Framework",
            "tags": ["programming"],
            "options": [...]
        }
    ]
}
```

#### Get Poll by ID

* **URL:** `/polls/{id}`
* **Method:** `GET`
* **Responses:**
* **200 OK:**
```json
{
    "code": 200,
    "messages": "Poll retrieved successfully",
    "data": { 
        "id": "550e8400-e29b-41d4-a716-446655440000", 
        "title": "Favorite Backend Framework", 
        "tags": ["programming"], 
        "options": [...] 
    }
}
```

* **404 Not Found:**
```json
{
    "code": 404,
    "messages": "Poll not found",
    "data": null
}
```

#### Update Poll

* **URL:** `/polls/{id}`
* **Method:** `PUT`
* **Request Body:** Same structure as `PollInput` (fields to update).
* **Responses:**
* **200 OK:**
```json
{
    "code": 200,
    "messages": "Poll updated successfully",
    "data": null
}
```

* **404 Not Found:**
```json
{
    "code": 404,
    "messages": "Poll not found",
    "data": null
}
```

#### Delete Poll

* **URL:** `/polls/{id}`
* **Method:** `DELETE`
* **Responses:**
* **204 No Content:**
```json
{
    "code": 204,
    "messages": "Poll deleted successfully",
    "data": null
}
```

* **404 Not Found:**
```json
{
    "code": 404,
    "messages": "Poll not found",
    "data": null
}
```

---

### 2. Search and Filtering

#### Search Polls by Title

* **URL:** `/polls/search`
* **Method:** `GET`
* **Query Parameters:** `?query={keyword}`
* **Responses:**
* **200 OK (Match found or empty query fallback):**
```json
{
    "code": 200,
    "messages": "Search completed successfully",
    "data": [...]
}
```

* **400 Bad Request (Missing Query Parameter structure):**
```json
{
    "code": 400,
    "messages": "Search query parameter is required",
    "data": null
}
```

#### Filter Polls by Tags

* **URL:** `/polls/filter`
* **Method:** `POST`
* **Request Body (JSON array of strings):**
```json
["programming", "tech"]
```

* **Response (200 OK):**
```json
{
    "code": 200,
    "messages": "Filter applied successfully",
    "data": [...]
}
```

#### Get All Registered Unique Tags

* **URL:** `/tags`
* **Method:** `GET`
* **Response (200 OK):**
```json
{
    "code": 200,
    "messages": "Tags retrieved successfully",
    "data": ["programming", "backend", "tech"]
}
```

---

### 3. Voting Actions

#### Upvote Poll Option

* **URL:** `/polls/{id}/upvote`
* **Method:** `PUT`
* **Request Body:**
```json
{
    "opt_id": "a1b2c3d4-e5f6-7a8b-9c0d-1e2f3a4b5c6d"
}
```

* **Responses:**
* **200 OK:** Returns the updated parent Poll object with the incremented vote value.
* **400 Bad Request:** Occurs if `opt_id` is a nil UUID or not associated with the poll.

#### Downvote Poll Option

* **URL:** `/polls/{id}/downvote`
* **Method:** `PUT`
* **Request Body:**
```json
{
    "opt_id": "a1b2c3d4-e5f6-7a8b-9c0d-1e2f3a4b5c6d"
}
```

* **Responses:**
* **200 OK:** Returns the updated parent Poll object with the decremented vote value (clamped at 0).

---

### 4. Global Error Fallback

#### Unregistered URL Route Target

* **URL:** Any unregistered pattern (e.g., `/invalid-endpoint`)
* **Method:** Any
* **Response (404 Not Found):**
```json
{
    "code": 404,
    "messages": "Route '/invalid-endpoint' is not registered on this server",
    "data": null
}
```

---

## License

This project is licensed under the terms of the MIT License. You can review the full text of the terms and conditions in the accompanying `LICENSE` file located in the root folder directory.