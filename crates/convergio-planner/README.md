# convergio-planner

Layer 4 reference planner.

`Planner::solve` turns a newline-separated mission into one local plan
with one task per non-empty line. It is deterministic and deliberately
small: enough for a local quickstart, easy to replace with your own
client over the HTTP API.
