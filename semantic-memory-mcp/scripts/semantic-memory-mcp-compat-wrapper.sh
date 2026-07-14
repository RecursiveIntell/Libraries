#!/usr/bin/env bash
# Compatibility guard for legacy MCP configurations.
#
# Old clients invoked ~/.cargo/bin/semantic-memory-mcp directly, which made a
# new model/database owner per session. Keep that path as a tiny relay so old
# watchdogs cannot recreate the heavyweight process while configurations roll
# over. The full profile is explicitly routed to the admin listener.
set -euo pipefail

profile=agent
expect_profile=0
for arg in "$@"; do
  if [ "$expect_profile" = 1 ]; then
    profile=$arg
    break
  fi
  [ "$arg" = "--tool-profile" ] && expect_profile=1
done

case "$profile" in
  full) port=17541 ;;
  *) port=17540 ;;
esac

exec python3 /home/sikmindz/Coding/Libraries/semantic-memory-mcp/scripts/semantic-memory-mcp-relay.py --port "$port"
