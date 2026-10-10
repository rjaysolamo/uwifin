#!/usr/bin/env bash
set -euo pipefail
umask 077
# Credentials belong in a private MariaDB client option file, never command arguments.
: "${DB_CLIENT_CONFIG:?Set DB_CLIENT_CONFIG to a private MariaDB client .cnf file}"
: "${BACKUP_DIR:?Set BACKUP_DIR to a private backup directory}"
mkdir -p "$BACKUP_DIR"
backup_file="$BACKUP_DIR/uwifin-$(date -u +%Y%m%dT%H%M%SZ).sql.gz"
trap 'rm -f "$backup_file.tmp"' EXIT
mariadb-dump --defaults-extra-file="$DB_CLIENT_CONFIG" --single-transaction --routines --events --triggers --hex-blob "${DB_NAME:-uwifin}" | gzip > "$backup_file.tmp"
gzip -t "$backup_file.tmp"
mv "$backup_file.tmp" "$backup_file"
find "$BACKUP_DIR" -maxdepth 1 -name 'uwifin-*.sql.gz' -type f -mtime +14 -delete
