# Crawl CLI (`crawl`)

The `crawl` CLI connects to `crawl-sysd` over a single Unix socket and sends JSON-RPC commands. It covers all daemon-managed services.

## Socket Architecture

```
┌──────────┐   single socket   ┌──────────────┐
│  Client  │ ──────────────────►  crawl-sysd   │
│ (CLI/QML)│   $runtime_dir/   │  (IPC router) │
└──────────┘    crawl.sock      └──────┬───────┘
                                       │
                          ┌────────────┼────────────┐
                          ▼            ▼            ▼
                   ┌──────────┐ ┌──────────┐ ┌──────────┐
                   │ Local    │ │crawl-    │ │crawl-    │
                   │ Services │ │webservice│ │mail      │
                   │ (audio,  │ │(RSS,     │ │(IMAP,    │
                   │  network,│ │ Wallhaven)│ │ SMTP)    │
                   │  ...)    │ │          │ │          │
                   └──────────┘ └──────────┘ └──────────┘
```

`crawl-sysd` exposes a single Unix socket (`$XDG_RUNTIME_DIR/crawl.sock`). All clients connect here. Commands for child daemons (RSS, Wallhaven, Mail) are forwarded automatically by the built-in `IpcRouter`. Events from child daemons are bridged back so subscribers see everything on one connection.

## Usage

```bash
crawl <command> [args...]
crawl --json <command>   # machine-readable JSON output
```

## Commands

| Command | Description | Routes to |
|---------|-------------|-----------|
| `audio` | Volume, sinks, sources | Local |
| `bluetooth` | Devices, scan, pair, power | Local |
| `brightness` | Get/set backlight | Local |
| `daemon` | Lifecycle and diagnostics | Local |
| `health` | Service health check | Local |
| `network` | Wi-Fi, Ethernet, hotspot | Local |
| `proc` | Process list, find, kill | Local |
| `shell` | Interactive REPL | Local |
| `status` | Overall daemon status | Local |
| `sysmon` | CPU, memory, disk, network monitoring | Local |
| `sysinfo` | System information | Local |
| `theme` | Get, list, set, generate themes | Local |

RSS and Mail CLI commands are planned — in the meantime, use `crawl shell` with raw JSON-RPC or the QML Settings panel.

## Configuration

The socket path defaults to `$XDG_RUNTIME_DIR/crawl.sock` and can be overridden with `CRAWL_SOCKET`:

```bash
CRAWL_SOCKET=/tmp/crawl.sock crawl status
```

## Event Subscriptions

Use `crawl shell` with `--subscribe` to receive live events from all services (including forwarded child daemon events):

```bash
echo '{"jsonrpc":"2.0","method":"Subscribe","params":{"topics":[]},"id":1}' | crawl shell
```

## Adding CLI Commands for Routed Services

To add CLI subcommands for services behind the router (RSS, Mail, etc.):

1. Define the command in `crawl-cli/src/cmd/`
2. Add the `CrawlCommand` variant to `crawl-ipc/src/commands.rs` (these already exist for RSS and Mail)
3. Register the subcommand in `crawl-cli/src/main.rs`
4. The router in `crawl-sysd` handles forwarding — no CLI-side changes are needed for routing

## Troubleshooting

| Symptom | Likely Cause |
|---------|-------------|
| `failed to connect to crawl daemon` | `crawl-sysd` not running (`systemctl --user start crawl`) |
| `Child daemon error: ... unavailable` | Child daemon (webservice/mail) not running or socket wrong |
| `Unknown method` | Command not recognized by any daemon; check `crawl-ipc` command definitions |
| Events from mail/webservice not appearing | Event bridge in sysd reconnects every 5s; check child daemon is alive |
