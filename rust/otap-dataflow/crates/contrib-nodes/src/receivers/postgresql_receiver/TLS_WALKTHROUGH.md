# PostgreSQL Receiver TLS Walkthrough

This Windows PowerShell walkthrough uses Docker Desktop to create a disposable
PostgreSQL 15.19 server with a generated private CA and a certificate valid for
`localhost`. Run all commands from `rust/otap-dataflow`.

<!-- markdownlint-disable MD013 -->

## Start PostgreSQL with TLS

```powershell
$container = 'otel-postgresql-tls'
$postgres = @'
set -eu
mkdir -p /var/lib/postgresql/tls
openssl req -x509 -newkey rsa:2048 -nodes -days 2 -subj '/CN=OTel PostgreSQL Test CA' -keyout /var/lib/postgresql/tls/ca.key -out /var/lib/postgresql/tls/ca.crt
openssl req -newkey rsa:2048 -nodes -subj '/CN=localhost' -addext 'subjectAltName=DNS:localhost,IP:127.0.0.1' -keyout /var/lib/postgresql/tls/server.key -out /var/lib/postgresql/tls/server.csr
openssl x509 -req -in /var/lib/postgresql/tls/server.csr -CA /var/lib/postgresql/tls/ca.crt -CAkey /var/lib/postgresql/tls/ca.key -CAcreateserial -days 2 -copy_extensions copyall -out /var/lib/postgresql/tls/server.crt
chown -R postgres:postgres /var/lib/postgresql/tls
chmod 600 /var/lib/postgresql/tls/server.key
exec docker-entrypoint.sh postgres -c ssl=on -c ssl_cert_file=/var/lib/postgresql/tls/server.crt -c ssl_key_file=/var/lib/postgresql/tls/server.key
'@

docker run --name $container --detach --publish 5432:5432 `
  --env POSTGRES_PASSWORD=postgres-admin `
  --env POSTGRES_DB=receiver_fixture `
  postgres:15.19 bash -ceu $postgres

for ($attempt = 0; $attempt -lt 30; $attempt++) {
  docker exec $container pg_isready -U postgres -d receiver_fixture
  if ($LASTEXITCODE -eq 0) { break }
  Start-Sleep -Seconds 1
}
if ($LASTEXITCODE -ne 0) { throw 'PostgreSQL did not become ready' }
```

## Create the source data and reader

```powershell
$sql = @'
CREATE ROLE otel_reader LOGIN PASSWORD 'otel-reader-password';
CREATE TABLE public.receiver_events (
  event_ts timestamptz(6) NOT NULL,
  event_id bigint NOT NULL,
  actor text NOT NULL,
  PRIMARY KEY (event_ts, event_id)
);
INSERT INTO public.receiver_events VALUES
  ('2026-10-05T00:00:01Z', 1, 'alice'),
  ('2026-10-05T00:00:01Z', 2, 'bob'),
  ('2026-10-05T00:00:02.123456Z', 3, 'carol');
GRANT CONNECT ON DATABASE receiver_fixture TO otel_reader;
GRANT USAGE ON SCHEMA public TO otel_reader;
GRANT SELECT ON public.receiver_events TO otel_reader;
'@

$sql | docker exec -i $container psql -v ON_ERROR_STOP=1 `
  -U postgres -d receiver_fixture
```

## Configure credentials and trust

Export the private CA and create credential files without putting credentials in
YAML:

```powershell
New-Item -ItemType Directory -Force .secrets\postgresql | Out-Null
Set-Content .secrets\postgresql\username 'otel_reader' -Encoding ascii
Set-Content .secrets\postgresql\password 'otel-reader-password' -Encoding ascii
docker cp "${container}:/var/lib/postgresql/tls/ca.crt" `
  .secrets\postgresql\ca.crt

$env:PG_HOST = 'localhost'
$env:PG_USERNAME_FILE = (Resolve-Path .secrets\postgresql\username).Path
$env:PG_PASSWORD_FILE = (Resolve-Path .secrets\postgresql\password).Path
$env:PG_CA_FILE = (Resolve-Path .secrets\postgresql\ca.crt).Path
```

## Run and verify the receiver

Ensure the engine state directory used by `checkpoint.directory` is writable
and persisted across receiver or host restarts, then run:

```powershell
cargo run --no-default-features --features postgresql,crypto-ring `
  --bin df_engine -- --config configs/postgresql-console.yaml
```

No OTLP endpoint is needed. The three source rows appear in the console output:

```text
RESOURCE v1.Resource [db.system.name=postgresql, receiver.database.source_id=orders-audit]
INFO database.query.row: {event_ts=2026-10-05T00:00:01.000000Z, event_id=1, actor=alice}
INFO database.query.row: {event_ts=2026-10-05T00:00:01.000000Z, event_id=2, actor=bob}
INFO database.query.row: {event_ts=2026-10-05T00:00:02.123456Z, event_id=3, actor=carol}
```

While the receiver is running, open a second PowerShell terminal and verify the
live connection is encrypted:

```powershell
$container = 'otel-postgresql-tls'
docker exec $container psql -U postgres -d receiver_fixture -c `
  "SELECT usename, ssl, version, cipher FROM pg_stat_ssl JOIN pg_stat_activity USING (pid) WHERE usename = 'otel_reader';"
```

The result must show `ssl` as `t`.

## Clean up

Stop the receiver with Ctrl+C, then remove the disposable database, credentials,
CA, and checkpoint state:

```powershell
docker rm --force $container
Remove-Item -Recurse -Force .secrets\postgresql
Remove-Item -Recurse -Force .otap-state\postgresql -ErrorAction SilentlyContinue
```
