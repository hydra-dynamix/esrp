# ESRP Test Server

A local test server implementing the ESRP protocol. Can run in mock mode or proxy to real Erasmus services.

## Quick Start

```bash
# Run in mock mode (default)
cargo run --package esrp-server

# Run in proxy mode (forward to real Erasmus services)
ERASMUS_URL=http://localhost:8000 cargo run --package esrp-server
```

## Endpoints

- `GET /v1/health` - Health check
- `POST /v1/execute` - Main ESRP execute endpoint (routes based on target.service)
- `POST /v1/translate` - Convenience endpoint for translation
- `POST /v1/tts` - Convenience endpoint for text-to-speech
- `POST /v1/stt` - Convenience endpoint for speech-to-text

## Testing

### Health Check
```bash
curl http://localhost:8080/v1/health
```

### Translation (Mock)
```bash
curl -X POST http://localhost:8080/v1/execute \
  -H "Content-Type: application/json" \
  -d @fixtures/v1/requests/simple_tts.json \
  | jq '.target.service = "translator" | .target.operation = "translate"'
```

### Text-to-Speech (Mock)
```bash
curl -X POST http://localhost:8080/v1/execute \
  -H "Content-Type: application/json" \
  -d @fixtures/v1/requests/simple_tts.json
```

### Using the ESRP CLI Client
```bash
# Start the server in one terminal
cargo run --package esrp-server

# Use the esrp-http client from another terminal (in your code)
use esrp_http::ESRPClient;

let client = ESRPClient::new("http://localhost:8080");
let response = client.execute(request).await?;
```

## Mock Behavior

In mock mode, the server returns simulated responses:

- **translator**: Returns `[LANG] original_text` where LANG is the target language
- **tts**: Returns a minimal valid WAV file header (silent audio)
- **stt**: Returns a placeholder transcription
- **video**: Returns `accepted` status with a job ID (async mock)

## Proxy Mode

Set `ERASMUS_URL` to forward requests to real Erasmus services:

```bash
ERASMUS_URL=http://localhost:8000 cargo run --package esrp-server
```

The server will translate ESRP requests to the legacy Erasmus format and convert responses back.
