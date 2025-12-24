# Ticket 802: Create Pydantic Models

**Phase:** 8 - Python Bindings
**Priority:** Medium
**Dependencies:** [801 - Create PyO3 Bindings](801-create-pyo3-bindings.md)
**Blocked By:** Ticket 801

## Summary

Create Pydantic models matching all ESRP types for Python usage.

## Implementation Details

### bindings/python/esrp/models.py

```python
from pydantic import BaseModel, Field
from uuid import UUID, uuid4
from datetime import datetime
from typing import Any, Literal, Optional
from enum import Enum

class Caller(BaseModel):
    system: str = "erasmus"
    agent_id: Optional[str] = None
    run_id: Optional[str] = None

class Target(BaseModel):
    service: str
    operation: str
    variant: Optional[str] = None

class ModeType(str, Enum):
    SYNC = "sync"
    ASYNC = "async"

class Mode(BaseModel):
    type: ModeType = ModeType.SYNC
    timeout_ms: int = 600000

class Input(BaseModel):
    name: str
    content_type: str
    data: str
    encoding: str = "utf-8"
    metadata: dict = Field(default_factory=dict)

class ESRPRequest(BaseModel):
    esrp_version: str = "1.0"
    request_id: UUID = Field(default_factory=uuid4)
    timestamp: datetime = Field(default_factory=datetime.utcnow)
    caller: Caller
    target: Target
    mode: Mode = Field(default_factory=Mode)
    inputs: list[Input]
    params: dict = Field(default_factory=dict)
```

## Acceptance Criteria

- [ ] All ESRP types have Pydantic models
- [ ] Models validate correctly
- [ ] Can serialize/deserialize JSON
