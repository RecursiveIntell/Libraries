#!/usr/bin/env python3
"""Forward one stdio MCP client to the single local semantic-memory owner.

This program intentionally contains no store, model, or tool logic. Configure
MCP clients to execute it in place of semantic-memory-mcp.
"""

import argparse
import os
import selectors
import socket
import sys


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--port", type=int, required=True)
    args = parser.parse_args()

    try:
        upstream = socket.create_connection(("127.0.0.1", args.port), timeout=5)
    except OSError as error:
        print(f"semantic-memory relay: daemon unavailable on port {args.port}: {error}", file=sys.stderr)
        return 1

    upstream.setblocking(False)
    stdin_fd = sys.stdin.buffer.fileno()
    stdout_fd = sys.stdout.buffer.fileno()
    selector = selectors.DefaultSelector()
    selector.register(stdin_fd, selectors.EVENT_READ, "stdin")
    selector.register(upstream, selectors.EVENT_READ, "upstream")

    try:
        while True:
            for key, _ in selector.select():
                if key.data == "stdin":
                    data = os.read(stdin_fd, 65536)
                    if not data:
                        upstream.shutdown(socket.SHUT_WR)
                        selector.unregister(stdin_fd)
                        continue
                    upstream.sendall(data)
                else:
                    data = upstream.recv(65536)
                    if not data:
                        return 0
                    os.write(stdout_fd, data)
    finally:
        selector.close()
        upstream.close()


if __name__ == "__main__":
    raise SystemExit(main())
