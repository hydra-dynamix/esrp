# Ticket 804: Create FastAPI Middleware

**Phase:** 8 - Python Bindings
**Priority:** Medium
**Dependencies:** [802 - Create Pydantic Models](802-create-pydantic-models.md)
**Blocked By:** Ticket 802

## Summary

Create FastAPI middleware for ESRP services.

## Implementation Details

### bindings/python/esrp/fastapi.py

```python
from fastapi import Request, HTTPException
from fastapi.responses import JSONResponse
from .models import ESRPRequest, ESRPResponse
import esrp_py

async def esrp_middleware(request: Request) -> ESRPRequest:
    """Extract and validate ESRP request."""
    body = await request.json()
    esrp_req = ESRPRequest(**body)

    # Validate using Rust
    try:
        esrp_py.validate_request(esrp_req.model_dump_json())
    except ValueError as e:
        raise HTTPException(status_code=400, detail=str(e))

    return esrp_req

def esrp_response(response: ESRPResponse) -> JSONResponse:
    """Convert ESRP response to FastAPI response."""
    status_code = {
        "succeeded": 200,
        "accepted": 202,
        "failed": 500,
    }[response.status]

    return JSONResponse(
        content=response.model_dump(mode='json'),
        status_code=status_code,
    )
```

## Acceptance Criteria

- [ ] Middleware extracts ESRP requests
- [ ] Validation uses Rust implementation
- [ ] Response helper maps status codes
