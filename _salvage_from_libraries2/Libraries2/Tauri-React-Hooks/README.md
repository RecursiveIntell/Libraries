# @tauri-hooks/core

React hooks for Tauri 2 applications.

`@tauri-hooks/core` removes the repetitive parts of wiring a React frontend to Tauri commands and events: event subscription, query state, mutation state, config persistence, and high-frequency stream buffering.

## Hooks

| Hook | Purpose |
| --- | --- |
| `useTauriEvent` | Subscribe to one event with fresh handlers and safe cleanup |
| `useTauriEvents` | Subscribe to multiple events at once |
| `useTauriQuery` | Run a command and manage `data/loading/error/refresh` |
| `useTauriMutation` | Wrap a command as an explicit mutation |
| `useTauriConfig` | Load, update, save, and reload a config object |
| `useBufferedStream` | Batch high-frequency text/data updates into controlled renders |

## Install

```bash
npm install @tauri-hooks/core
```

Peer dependencies:

- `react >= 18`
- `@tauri-apps/api >= 2`

## Quick Examples

### Event subscription

```tsx
import { useTauriEvent } from "@tauri-hooks/core";

function QueueMonitor() {
  useTauriEvent<{ jobId: string }>("queue:job_completed", (payload) => {
    console.log(payload.jobId);
  });

  return null;
}
```

### Query

```tsx
import { useTauriQuery } from "@tauri-hooks/core";

const { data, loading, error, refresh } = useTauriQuery<string[]>(
  "list_images",
  { folder: "/tmp/gallery" },
  { refreshOn: ["queue:job_completed"] },
);
```

### Mutation

```tsx
import { useTauriMutation } from "@tauri-hooks/core";

const { mutate, loading } = useTauriMutation<[string], void>(
  "delete_image",
  (path) => ({ path }),
);
```

### Config

```tsx
import { useTauriConfig } from "@tauri-hooks/core";

const { config, update, save, reload } = useTauriConfig<AppConfig>(
  "get_config",
  "save_config",
);
```

### Buffered stream

`useBufferedStream` is intentionally manual during the active stream lifecycle: call `start()` when streaming begins and `stop()` when it ends. Unmount cleanup is automatic.

```tsx
import { useBufferedStream, useTauriEvent } from "@tauri-hooks/core";

const stream = useBufferedStream({ interval: 33 });

useTauriEvent<{ streamId: string; token: string }>("llm:token", (payload) => {
  stream.push(payload.streamId, payload.token);
});
```

## Usage Notes

- `useTauriEvents` re-subscribes when its `deps` change. If event bindings themselves are dynamic, memoize them or include the right dependencies.
- `useBufferedStream` still expects explicit `start()` / `stop()` around the active stream lifecycle, but it now clears any active interval automatically on unmount.

## Why This Package Exists

This package is the frontend counterpart to crates like `tauri-queue` and `ai-batch-queue`. It gives you a small, reusable UI layer instead of rewriting async/event plumbing in every app.

## License

MIT
