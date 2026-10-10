#!/usr/bin/env bash
set -euo pipefail
: "${DB_CLIENT_CONFIG:?Set DB_CLIENT_CONFIG to the disposable restore server client config}"
: "${RESTORE_DATABASE:?Set a disposable database name beginning uwifin_restore_}"
[[ "$RESTORE_DATABASE" =~ ^uwifin_restore_[a-zA-Z0-9_]+$ ]] || { echo 'Use a disposable uwifin_restore_ database.' >&2; exit 1; }
backup_file="${1:?Pass the backup .sql.gz file}"
gzip -t "$backup_file"
mariadb --defaults-extra-file="$DB_CLIENT_CONFIG" -e "CREATE DATABASE \`$RESTORE_DATABASE\`;"
gzip -dc "$backup_file" | mariadb --defaults-extra-file="$DB_CLIENT_CONFIG" "$RESTORE_DATABASE"
mariadb --defaults-extra-file="$DB_CLIENT_CONFIG" "$RESTORE_DATABASE" -e 'SELECT COUNT(*) AS users FROM users; SELECT COUNT(*) AS transactions FROM transactions; CHECK TABLE users,sessions,wallets,transactions,payments,audit_events;'
