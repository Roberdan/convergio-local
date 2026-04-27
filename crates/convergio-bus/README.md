# convergio-bus

Layer 2 of Convergio — persistent agent message bus.

## Status

**Implemented.** Topic-based publish, polling consumer with cursor,
explicit ack. Scoped per `plan_id`. Persistent via SQLite.

## API

| Op | Function |
|----|----------|
| Publish | `Bus::publish(NewMessage { plan_id, topic, sender, payload })` |
| Poll | `Bus::poll(plan_id, topic, cursor, limit) -> Vec<Message>` |
| Ack | `Bus::ack(message_id, consumer)` |

HTTP surface (mounted by `convergio-server`):

| Method | Path |
|--------|------|
| `POST` | `/v1/plans/:plan_id/messages` |
| `GET`  | `/v1/plans/:plan_id/messages?topic=&cursor=&limit=` |
| `POST` | `/v1/messages/:id/ack` |

## Delivery semantics

- **At-least-once** — consumer must be idempotent.
- **Persistent** — messages survive consumer crash until acked.
- **Per-`(plan_id, topic)` FIFO** ordered by `seq`.

## What it is NOT

- Cross-plan or system-wide broadcast.
- Sub-millisecond throughput (Kafka territory).
- Content-aware routing — payload is opaque JSON.
