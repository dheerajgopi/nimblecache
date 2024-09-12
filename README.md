# Nimblecache

Nimblecache is an in-memory database written in Rust that supports the Redis Serialization Protocol (RESP).
It's a hobby project of mine as part of learning Rust, where I'm using CodeCrafter's '[Build Your Own Redis](https://app.codecrafters.io/courses/redis/overview)'
as a reference.

## Getting Started

### Prerequisites

- Rust (MSRV >= 1.75.0)
- Cargo

### Run the Nimblecache server

Run `make run-dev` to run the Nimblecache server on port 6379.

## Supported Redis Commands:

- PING
- INFO (Partial)
- SET (Without expiry)
- GET
- LPUSH
- RPUSH
- LRANGE
- MULTI
- EXEC
- DISCARD
