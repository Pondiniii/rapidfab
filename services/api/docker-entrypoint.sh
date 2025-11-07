#!/bin/sh
set -e

echo "Waiting for PostgreSQL..."

# Wait for postgres to be ready
until pg_isready -h postgres -p 5432 -U rapidfab > /dev/null 2>&1; do
  echo "PostgreSQL is unavailable - sleeping"
  sleep 2
done

echo "PostgreSQL is up - running migrations"

# Run migrations
# Note: migrations are embedded in the binary via sqlx::migrate!()
# so they run automatically on startup in main.rs
# This is just a health check

echo "Starting API..."
exec "$@"
