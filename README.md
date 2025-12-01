# Mirage API

Blazing-fast **API mock server** written in Rust.

Point it to a YAML config or OpenAPI spec — get a fully functional mock server with
latency, flaky responses and templated bodies.

> Feed it YAML → get a mock backend instantly.

⚠️ Project is in active development.  
🚧 MVP in progress.

## Features (MVP)

- 🧩 Describe routes in simple **YAML**
- 🚀 Fast single-binary server (Rust, tokio, axum)
- 🎯 Path & query params templating in responses
- 🐢 Configurable **latency** per route
- 🎲 Flaky responses: random errors with configured probability

## Running

cargo run -- --config examples/openapi.yaml --addr 127.0.0.1:8080

![Build](https://img.shields.io/github/actions/workflow/status/babikov/mirage-api/ci.yml)
![License](https://img.shields.io/badge/license-MIT-blue.svg)
![Stars](https://img.shields.io/github/stars/babikov/mirage-api)