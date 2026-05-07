# remailable

An email client for the [reMarkable Paper Pro](https://remarkable.com/) that renders beautifully on e-ink.

<img src="docs/screenshots/account-list.png" width="400" />
<img src="docs/screenshots/email-reader.png" width="400" />

## What it does

remailable brings your IMAP inbox to your reMarkable Paper Pro with a paper-like reading experience:

- **IMAP sync** — fetches your latest emails from INBOX via raw IMAP commands
- **HTML email rendering** — extracts and renders HTML bodies from multipart MIME messages, with e-ink-optimized styling
- **Paper-like UI** — warm paper background, large fonts, rectangular outline buttons designed for the 300 PPI e-ink display

## How it runs

remailable uses **AppLoad**, xochitl's internal plugin system. It ships as:

- A **headless Rust backend** that handles IMAP connections (GMX, Gmail, any IMAP server)
- A **QML frontend** rendered inside xochitl

The backend and frontend communicate over a SEQPACKET socket. No external web servers, no Electron — it runs natively on the device's Cortex-A53.

```
┌──────────────┐     SEQPACKET     ┌──────────────────┐
│  QML frontend │ ◄──────────────► │  Rust backend     │
│  (xochitl)    │                  │  (IMAP + SQLite)  │
└──────────────┘                   └──────────────────┘
```

## Screenshots

### Account list
![Account list](docs/screenshots/account-list.png)

### Email reader
![Email reader](docs/screenshots/email-reader.png)

### Folder list
![Folder list](docs/screenshots/folder-list.png)

## Development

### Build

```bash
# Cross-compile the backend for reMarkable Paper Pro (aarch64)
bash scripts/docker-build.sh

# Compile QML resources
/opt/codex/rm-ferrari/sysroots/x86_64-codexsdk-linux/usr/libexec/rcc --binary -o packaging/resources.rcc qml/frontend/application.qrc

# Package for AppLoad
bash scripts/package.sh

# Deploy to device
scp build/remailable-appload.tar.gz root@<device-ip>:/tmp/
ssh root@<device-ip> 'rm -rf /home/root/xovi/exthome/appload/remailable && cd /tmp && tar xzf remailable-appload.tar.gz && mv remailable /home/root/xovi/exthome/appload/remailable'
```

### Run locally

Set `REMAILABLE_NO_QT=1` to build the backend without Qt:

```bash
REMAILABLE_NO_QT=1 cargo run --bin remailable-backend -- /tmp/remailable.sock
```

## Tech stack

- **Backend**: Rust with `imap` crate (raw IMAP commands via `run_command_and_read_response`)
- **Frontend**: Qt Quick (QML) rendered inside reMarkable's xochitl compositor
- **Storage**: SQLite via `rusqlite` for email metadata and account configuration
- **MIME**: Custom multipart parser with base64 and quoted-printable decoding

## License

MIT
