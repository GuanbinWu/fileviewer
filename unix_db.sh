set -euo pipefail

# 管理员账号，用来先创建用户和数据库
export PGHOST="localhost"
export PGPORT="5432"
export PGUSER="postgres"
export PGPASSWORD="postgres_admin_password"

DB_USER="fviewer"
DB_PASS="Fviewer#123"
DB_NAME="fviewerdb"

# 1) 创建用户
sudo -u postgres psql  -v ON_ERROR_STOP=1 <<SQL
CREATE ROLE ${DB_USER} WITH LOGIN PASSWORD '${DB_PASS}';
CREATE DATABASE ${DB_NAME} OWNER ${DB_USER};
SQL


# 2) 以新用户身份执行建表等操作
export PGUSER="$DB_USER"
export PGPASSWORD="$DB_PASS"

psql -d "$DB_NAME" -v ON_ERROR_STOP=1 <<'SQL'

CREATE TABLE IF NOT EXISTS events (
    id BIGSERIAL PRIMARY KEY,
    username TEXT NOT NULL,
    action TEXT NOT NULL,
    time TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT now(),
    status TEXT NOT NULL,
    filepath TEXT NOT NULL,
    args TEXT);

CREATE TABLE IF NOT EXISTS zones (id SERIAL PRIMARY KEY,name TEXT NOT NULL,lords JSONB);
ALTER TABLE zones ADD CONSTRAINT zones_unique UNIQUE (name);

CREATE TABLE IF NOT EXISTS files (
    id           BIGSERIAL NOT NULL PRIMARY KEY,
    name         JSONB NOT NULL,
    parent_name  JSONB,
    is_directory BOOLEAN NOT NULL,
    size         BIGINT NOT NULL,
    content_type TEXT NOT NULL,
    md5          TEXT,
    created_at   TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    modified_at  TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    creator      TEXT NOT NULL,
    last_modifier TEXT NOT NULL,
    zone TEXT NOT NULL);

CREATE TABLE IF NOT EXISTS accounts (
    username TEXT NOT NULL PRIMARY KEY,
    hashed   TEXT NOT NULL
);
SQL