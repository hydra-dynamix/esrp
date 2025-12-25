# Ticket 905: Update Python Services

**Phase:** 9 - Service Migration
**Priority:** Medium
**Dependencies:** [804 - Create FastAPI Middleware](804-create-fastapi-middleware.md)
**Blocked By:** Ticket 804

## Summary

Update any Python-based services to use ESRP via the Python bindings.

## Tasks

1. Import esrp package in services
2. Use ESRP middleware for request handling
3. Return ESRP responses

## Acceptance Criteria

- [ ] Python services use ESRP models
- [ ] Services validate via Rust bindings
- [ ] Tests updated
