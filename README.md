# 📘 Mirage API

### ⚡ Ultra-fast OpenAPI Mock Server powered by Rust + Axum

Mirage API is a lightweight, blazing-fast, zero-config mock server that
turns **any OpenAPI YAML** into a **fully functional API** with
examples, schemas, and automatic mock generation.

Perfect for:

-   Frontend developers
-   Mobile apps
-   QA teams
-   Integration testing
-   Local development without a backend

Just point it to your `openapi.yaml` --- Mirage does the rest.

------------------------------------------------------------------------

## 🚀 Features

-   🔥 Instant mocks from OpenAPI

-   🎯 Example selection via query parameter

    ``` yaml
    x-mirage-example-param: variant
    ```

    → `/login?variant=success`

-   ✨ Support for `example:` and `examples:`

-   🧬 Schema-based mock generation

-   🛣 Path templates like `/users/{id}`

-   ⚡ Built with Rust + Axum

-   🧵 Zero configuration

------------------------------------------------------------------------

## 📦 Installation

### Build locally

``` bash
git clone https://github.com/babikov/mirage-api
cd mirage-api
cargo build --release
```

### Run

``` bash
mirage-api --config openapi.yaml --addr 127.0.0.1:8080
```

------------------------------------------------------------------------

## 🧪 Example OpenAPI

``` yaml
openapi: "3.0.0"
info:
  title: Mirage Example
  version: 1.0.0

paths:
  /login:
    get:
      summary: Login mock
      responses:
        "200":
          description: OK
          content:
            application/json:
              x-mirage-example-param: variant
              examples:
                success:
                  value:
                    status: "ok"
                    token: "jwt.token.string"
                error:
                  value:
                    status: "error"
                    message: "Invalid credentials"
```

### Example requests

``` bash
curl http://127.0.0.1:8080/login?variant=success
curl http://127.0.0.1:8080/login?variant=error
curl http://127.0.0.1:8080/login
```

------------------------------------------------------------------------

## 🧬 Schema-driven mock generation

If no examples are provided, Mirage generates JSON automatically:

``` yaml
schema:
  type: object
  properties:
    id:
      type: integer
    name:
      type: string
    tags:
      type: array
      items:
        type: string
```

Generated output:

``` json
{
  "id": 123,
  "name": "string",
  "tags": ["string"]
}
```

Supports:

-   string
-   integer
-   number
-   boolean
-   array
-   object
-   enum
-   formats (`date-time`, `uuid`)

------------------------------------------------------------------------

## 🛣 Path parameters

Template:

``` yaml
/users/{id}
```

Matches:

    /users/12
    /users/abc
    /users/99999

------------------------------------------------------------------------

## 💡 Why Mirage API?

-   Minimalistic, predictable
-   No UI, no complexity
-   Rust performance
-   Perfect for rapid prototyping
-   100% OpenAPI-compatible

------------------------------------------------------------------------

## 📍 Roadmap

-   [ ] Response delays (`x-mirage-delay-ms`)
-   [ ] Flaky responses
-   [ ] Random example mode
-   [ ] Docker image
-   [ ] Homebrew tap
-   [ ] CLI generator (`mirage-api init`)
-   [ ] TypeScript type generator

------------------------------------------------------------------------

## 🤝 Contributing

PRs are welcome.
Please keep the code clean and idiomatic.

------------------------------------------------------------------------

## 📝 License

MIT License.

------------------------------------------------------------------------

## ⭐ If you like the project --- star it!

Stars help others discover Mirage API.

![Build](https://img.shields.io/github/actions/workflow/status/babikov/mirage-api/ci.yml)
![License](https://img.shields.io/badge/license-MIT-blue.svg)
![Stars](https://img.shields.io/github/stars/babikov/mirage-api)