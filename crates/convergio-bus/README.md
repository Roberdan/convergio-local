# convergio-bus

Layer 2 of Convergio — persistent agent message bus.

**Status: skeleton.** Public surface is provisional. See
[ROADMAP.md](../../ROADMAP.md) week 3-4 and
[ARCHITECTURE.md](../../ARCHITECTURE.md) for the intended shape.

## What it will do

- One row per published message in `agent_messages`
- Topic + direct messaging, scoped per plan
- Long-poll / SSE consumer protocol (no WebSocket in MVP)
- Ack on consume → exactly-once delivery for cooperative consumers
- Persistent by default so consumer restart does not lose messages

## What it will NOT do

- Cross-plan or system-wide messaging
- Sub-millisecond throughput (Kafka territory)
- Content interpretation — payload is opaque bytes/JSON
