# Echo App - Actix-Web Server

This is a simple **Echo App** built using **Actix-web** in Rust. 

The goal is to interact with a Database (could be cloud or self hosted) with the specified database (/info/db/schema.sql) to have a spotify-like client app.

## Getting Started
- All APIs are available in the /info/apis/echo-apis.* files

### Prerequisites

- Rust toolchain installed
- `cargo` available in your PATH
- watchexec tool installed (this is optional, but useful)

### Setup
1. Fill the .env.example file
2. Copy the example environment file:

```bash
cp .env.example .env
```

### Running the Server manually
1. tart the server using the provided script:
```bash
./start.sh
```

### Running the Server with Docker - Soon
