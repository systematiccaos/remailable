#!/bin/sh
# AppLoad backend entry point for remailable
# xochitl passes the SEQPACKET socket path as $1
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
exec "$SCRIPT_DIR/remailable-backend" "$1" 2>>/tmp/remailable-backend.log