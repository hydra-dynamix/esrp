# Ticket 803: Create Python Client

**Phase:** 8 - Python Bindings
**Priority:** Medium
**Dependencies:** [802 - Create Pydantic Models](802-create-pydantic-models.md)
**Blocked By:** Ticket 802

## Summary

Create Python HTTP client for ESRP services.

## Implementation Details

### bindings/python/esrp/client.py

```python
import httpx
from .models import ESRPRequest, ESRPResponse

class ESRPClient:
    def __init__(self, base_url: str, timeout: float = 30.0):
        self.base_url = base_url.rstrip('/')
        self.client = httpx.Client(timeout=timeout)

    def execute(self, request: ESRPRequest) -> ESRPResponse:
        response = self.client.post(
            f"{self.base_url}/v1/execute",
            json=request.model_dump(mode='json'),
        )
        response.raise_for_status()
        return ESRPResponse(**response.json())

    async def execute_async(self, request: ESRPRequest) -> ESRPResponse:
        async with httpx.AsyncClient(timeout=30.0) as client:
            response = await client.post(
                f"{self.base_url}/v1/execute",
                json=request.model_dump(mode='json'),
            )
            response.raise_for_status()
            return ESRPResponse(**response.json())
```

## Acceptance Criteria

- [ ] Sync client works
- [ ] Async client works
- [ ] Errors handled properly
