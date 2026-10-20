#!/usr/bin/env python3
# Copyright The OpenTelemetry Authors
# SPDX-License-Identifier: Apache-2.0

"""Provision an isolated, verified-TLS SQL Server 2022 development fixture."""

import argparse
import hashlib
import json
import os
from pathlib import Path
import secrets
import shutil
import subprocess
import sys
import time

IMAGE = (
    "mcr.microsoft.com/mssql/server@sha256:"
    "4402d880dd4c34bfa7d8705e56a86cd6c88da80a1f6bbbe741f999e76264a090"
)
LABEL = "otel-arrow.sql-server-fixture"
SQLCMD = "/opt/mssql-tools18/bin/sqlcmd"


def find_executable(name):
    executable = shutil.which(name)
    if executable:
        return executable
    if name == "openssl" and sys.platform == "win32":
        roots = []
        git = shutil.which("git")
        if git:
            roots.append(Path(git).resolve().parent.parent)
        for variable in ("ProgramW6432", "ProgramFiles", "ProgramFiles(x86)"):
            if os.environ.get(variable):
                roots.append(Path(os.environ[variable]) / "Git")
        if os.environ.get("LOCALAPPDATA"):
            roots.append(Path(os.environ["LOCALAPPDATA"]) / "Programs" / "Git")
        for root in roots:
            for relative in (("mingw64", "bin"), ("usr", "bin")):
                candidate = root.joinpath(*relative, "openssl.exe")
                if candidate.is_file():
                    return str(candidate)
    hint = (
        "Use OpenSSL on PATH or the OpenSSL bundled with Git for Windows."
        if name == "openssl"
        else "Install Docker Desktop and make its docker command available on PATH."
    )
    raise RuntimeError(f"Required executable '{name}' was not found. {hint}")


def run(args, *, env=None, timeout=60):
    try:
        result = subprocess.run(
            args, env=env, capture_output=True, text=True, timeout=timeout, check=False
        )
    except FileNotFoundError:
        raise RuntimeError(
            f"Unable to start '{Path(args[0]).name}': executable not found. Check its installation and PATH."
        ) from None
    if result.returncode:
        # Command arguments/output can contain fixture secrets. Keep diagnostics local.
        raise RuntimeError(f"{args[0]} failed with exit code {result.returncode}")
    return result.stdout.strip()


def server_tls_configuration(allow_unencrypted=False):
    return (
        "[network]\n"
        "tlscert = /var/opt/mssql/tls/server.crt\n"
        "tlskey = /var/opt/mssql/tls/server.key\n"
        "tlsprotocols = 1.2\n"
        f"forceencryption = {0 if allow_unencrypted else 1}\n"
    )

def prepare_certificate_variants(directory, openssl):
    """Issue replacement and expired leaves before deleting the fixture CA key."""
    for variant in ("replacement", "expired"):
        run([openssl, "req", "-new", "-newkey", "rsa:2048", "-nodes",
             "-keyout", str(directory / f"{variant}.key"),
             "-out", str(directory / f"{variant}.csr"), "-subj", "/CN=localhost"])
    run([openssl, "x509", "-req", "-in", str(directory / "replacement.csr"),
         "-CA", str(directory / "ca.crt"), "-CAkey", str(directory / "ca.key"),
         "-CAcreateserial", "-out", str(directory / "replacement.crt"),
         "-days", "2", "-sha256", "-extfile", str(directory / "server.ext")])
    (directory / "ca-index.txt").write_text("", encoding="ascii")
    (directory / "ca-serial.txt").write_text("1000\n", encoding="ascii")
    (directory / "ca-sign.cnf").write_text(
        "[ca]\ndefault_ca=fixture\n[fixture]\n"
        f"database={directory.as_posix()}/ca-index.txt\n"
        f"serial={directory.as_posix()}/ca-serial.txt\n"
        f"new_certs_dir={directory.as_posix()}\n"
        f"certificate={directory.as_posix()}/ca.crt\n"
        f"private_key={directory.as_posix()}/ca.key\n"
        "default_md=sha256\npolicy=subject\n[subject]\ncommonName=supplied\n",
        encoding="utf-8",
    )
    run([openssl, "ca", "-batch", "-notext", "-config", str(directory / "ca-sign.cnf"),
         "-in", str(directory / "expired.csr"), "-out", str(directory / "expired.crt"),
         "-startdate", "20000101000000Z", "-enddate", "20010101000000Z",
         "-extfile", str(directory / "server.ext")])
    shutil.copyfile(directory / "server.crt", directory / "original.crt")
    shutil.copyfile(directory / "server.key", directory / "original.key")


def published_port(docker, container):
    address = run([docker, "port", container, "1433/tcp"])
    if not address.startswith("127.0.0.1:") or len(address.splitlines()) != 1:
        raise RuntimeError("Fixture must publish exactly one loopback-only SQL Server port")
    port = int(address.removeprefix("127.0.0.1:"))
    if not 1 <= port <= 65535:
        raise RuntimeError("Fixture published an invalid SQL Server port")
    return port


def start(directory, allow_unencrypted=False):
    openssl = find_executable("openssl")
    docker = find_executable("docker")
    try:
        server_os = run([docker, "info", "--format", "{{.OSType}}"])
    except (RuntimeError, subprocess.TimeoutExpired):
        raise RuntimeError(
            "Docker is unavailable. Start Docker Desktop in Linux-container mode and retry."
        ) from None
    if server_os != "linux":
        raise RuntimeError("The fixture requires Docker in Linux-container mode.")
    directory.mkdir(parents=True, exist_ok=False, mode=0o700)
    ca_key = directory / "ca.key"
    ca = directory / "ca.crt"
    key = directory / "server.key"
    csr = directory / "server.csr"
    cert = directory / "server.crt"
    extensions = directory / "server.ext"
    extensions.write_text(
        "basicConstraints=critical,CA:FALSE\n"
        "keyUsage=critical,digitalSignature,keyEncipherment\n"
        "extendedKeyUsage=serverAuth\n"
        "subjectAltName=DNS:localhost\n", encoding="ascii"
    )
    run([openssl, "req", "-x509", "-newkey", "rsa:2048", "-nodes",
         "-keyout", str(ca_key), "-out", str(ca), "-days", "2", "-sha256",
         "-subj", "/CN=OTAP SQL Server fixture CA",
         "-addext", "basicConstraints=critical,CA:TRUE"])
    run([openssl, "req", "-new", "-newkey", "rsa:2048", "-nodes",
         "-keyout", str(key), "-out", str(csr), "-subj", "/CN=localhost"])
    run([openssl, "x509", "-req", "-in", str(csr), "-CA", str(ca),
         "-CAkey", str(ca_key), "-CAcreateserial", "-out", str(cert),
         "-days", "2", "-sha256", "-extfile", str(extensions)])
    prepare_certificate_variants(directory, openssl)
    (directory / "mssql.conf").write_text(
        server_tls_configuration(allow_unencrypted), encoding="ascii"
    )
    admin_password = "Aa1!" + secrets.token_urlsafe(30)
    reader_password = "Aa1!" + secrets.token_urlsafe(30)
    password_file = directory / "reader-password.txt"
    password_file.write_text(reader_password, encoding="ascii")
    setup = directory / "setup.sql"
    setup.write_text(
        f"""CREATE LOGIN otel_reader WITH PASSWORD = '{reader_password}', CHECK_POLICY = OFF;
GO
CREATE DATABASE otap_fixture;
GO
USE otap_fixture;
CREATE USER otel_reader FOR LOGIN otel_reader;
CREATE TABLE dbo.OTAP_SQL_EVENTS (
    ID bigint NOT NULL PRIMARY KEY,
    EVENT_TS datetime2(7) NOT NULL,
    PAYLOAD nvarchar(4000) NULL,
    AMOUNT decimal(38,8) NULL,
    RAW varbinary(32) NULL,
    AT_TIME datetimeoffset(7) NULL
);
INSERT INTO dbo.OTAP_SQL_EVENTS VALUES
    (1, '2026-01-01T00:00:01.1234567', N'hello ' + NCHAR(960),
     123456789012345678901234567890.12345678, 0x0001FF,
     '2026-01-01T05:30:00.1234567+05:30'),
    (2, '2026-01-01T00:00:02.1234567', N'second', NULL, NULL, NULL),
    (3, '2026-01-01T00:00:03.1234567', N'third', -1.00000001, 0xAB,
     '2026-01-01T00:00:00-04:00');
CREATE UNIQUE INDEX event_order ON dbo.OTAP_SQL_EVENTS(EVENT_TS, ID);
GRANT SELECT ON dbo.OTAP_SQL_EVENTS TO otel_reader;
GO
""", encoding="ascii"
    )
    owner = hashlib.sha256(str(directory).encode()).hexdigest()
    name = "otel-sql-" + secrets.token_hex(5)
    environment = dict(os.environ, MSSQL_SA_PASSWORD=admin_password)
    startup = (
        "set -eu; "
        "install -d -m 700 -o mssql -g root /var/opt/mssql/tls; "
        "install -m 600 -o mssql -g root /fixture/server.key /var/opt/mssql/tls/server.key; "
        "install -m 644 -o mssql -g root /fixture/server.crt /var/opt/mssql/tls/server.crt; "
        "install -m 644 -o mssql -g root /fixture/mssql.conf /var/opt/mssql/mssql.conf; "
        "install -m 644 /fixture/ca.crt /usr/local/share/ca-certificates/otap-fixture.crt; "
        "update-ca-certificates >/dev/null; "
        "exec runuser -u mssql -- /opt/mssql/bin/sqlservr"
    )
    container = run([
        docker, "run", "--detach", "--name", name,
        "--label", f"{LABEL}={owner}", "--memory", "3g",
        "--publish", "127.0.0.1::1433",
        "--env", "ACCEPT_EULA=Y", "--env", "MSSQL_PID=Developer",
        "--env", "MSSQL_SA_PASSWORD", "--user", "0",
        "--mount", f"type=bind,source={directory},target=/fixture,readonly",
        "--entrypoint", "/bin/bash", IMAGE, "-c", startup,
    ], env=environment, timeout=180)
    metadata = {
        "container": container, "owner": owner, "image": IMAGE,
        "allow_unencrypted": allow_unencrypted,
    }
    state = directory / "fixture.json"
    state.write_text(json.dumps(metadata, indent=2), encoding="utf-8")
    try:
        # Admin setup also verifies the fixture certificate; no sqlcmd -C bypass.
        admin_env = dict(os.environ, SQLCMDPASSWORD=admin_password)
        command = [docker, "exec", "--env", "SQLCMDPASSWORD", container,
                   SQLCMD, "-S", "localhost", "-U", "sa", "-N", "-b", "-l", "2"]
        deadline = time.monotonic() + 120
        while True:
            probe = subprocess.run(command + ["-Q", "SELECT 1"],
                                   env=admin_env, capture_output=True, timeout=10,
                                   check=False)
            if probe.returncode == 0:
                break
            if time.monotonic() >= deadline:
                raise RuntimeError("SQL Server did not become ready within 120 seconds")
            time.sleep(1)
        run(command + ["-i", "/fixture/setup.sql"], env=admin_env)
        version = run(command + ["-h", "-1", "-W", "-Q",
                      "SET NOCOUNT ON; SELECT CAST(SERVERPROPERTY('ProductVersion') AS varchar(30))"],
                      env=admin_env)
        if not version.startswith("16."):
            raise RuntimeError("The fixture did not report SQL Server 2022")
        port = published_port(docker, container)
        metadata.update({
            "server_version": version,
            "env": {
                "OTAP_SQL_SERVER_HOST": "localhost",
                "OTAP_SQL_SERVER_PORT": str(port),
                "OTAP_SQL_SERVER_DATABASE": "otap_fixture",
                "OTAP_SQL_SERVER_USERNAME": "otel_reader",
                "OTAP_SQL_SERVER_PASSWORD_FILE": str(password_file),
                "OTAP_SQL_SERVER_CA_FILE": str(ca),
                "OTAP_SQL_SERVER_ALLOW_UNENCRYPTED": "1" if allow_unencrypted else "0",
                "OTAP_SQL_SERVER_FIXTURE_DIRECTORY": str(directory),
                "OTAP_SQL_SERVER_FIXTURE_PYTHON": sys.executable,
                "OTAP_SQL_SERVER_CERTIFICATE": "valid",
            },
        })
        state.write_text(json.dumps(metadata, indent=2), encoding="utf-8")
        # Keep the server key until stop so a deliberate container restart works.
        for filename in ["setup.sql", "ca.key"]:
            (directory / filename).unlink()
        print(str(state))
    except (RuntimeError, subprocess.TimeoutExpired, OSError, ValueError):
        stop(directory)
        raise


def owned_fixture(directory):
    metadata = json.loads((directory / "fixture.json").read_text(encoding="utf-8"))
    if metadata.get("stopped"):
        raise RuntimeError("Fixture is already stopped")
    identity = run(["docker", "inspect", metadata["container"], "--format",
                    '{{index .Config.Labels "' + LABEL + '"}}'])
    if identity != metadata["owner"]:
        raise RuntimeError("Refusing to remove a container not owned by this fixture")
    return metadata


def admin_query(metadata, query, *, allow_expired=False):
    # Password expansion stays inside the owned disposable container. The sole
    # bypass is readiness checking after deliberately installing an expired leaf.
    verify = " -C" if allow_expired else ""
    return run(["docker", "exec", metadata["container"], "/bin/bash", "-c",
                f'SQLCMDPASSWORD="$MSSQL_SA_PASSWORD" {SQLCMD} -S localhost -U sa -N{verify} -b -l 2 -Q "$1"',
                "fixture-admin", query], timeout=10)


def rotate_certificate(directory, variant):
    if variant not in ("valid", "replacement", "expired"):
        raise ValueError("Unknown certificate variant")
    metadata = owned_fixture(directory)
    name = "original" if variant == "valid" else variant
    shutil.copyfile(directory / f"{name}.crt", directory / "server.crt")
    shutil.copyfile(directory / f"{name}.key", directory / "server.key")
    run(["docker", "restart", metadata["container"]], timeout=60)
    deadline = time.monotonic() + 120
    while True:
        try:
            admin_query(metadata, "SELECT 1", allow_expired=variant == "expired")
            break
        except (RuntimeError, subprocess.TimeoutExpired):
            if time.monotonic() >= deadline:
                raise RuntimeError("Rotated fixture did not become ready in 120 seconds") from None
            time.sleep(1)
    # Docker may allocate a new ephemeral host port on restart.
    metadata["env"]["OTAP_SQL_SERVER_PORT"] = str(published_port("docker", metadata["container"]))
    metadata["env"]["OTAP_SQL_SERVER_CERTIFICATE"] = variant
    (directory / "fixture.json").write_text(json.dumps(metadata, indent=2), encoding="utf-8")
    print(f"Installed {variant} fixture certificate; global trust unchanged")


def disconnect_readers(directory):
    metadata = owned_fixture(directory)
    # Never target the setup/admin session or unrelated clients. This operation
    # is explicit test fault injection, not an operator recovery shortcut.
    admin_query(metadata, """
DECLARE @session int, @command nvarchar(40);
DECLARE readers CURSOR LOCAL FAST_FORWARD FOR
SELECT session_id FROM sys.dm_exec_sessions
WHERE login_name = N'otel_reader' AND program_name = N'otel-arrow-sql-server'
  AND database_id = DB_ID(N'otap_fixture') AND session_id <> @@SPID;
OPEN readers;
FETCH NEXT FROM readers INTO @session;
WHILE @@FETCH_STATUS = 0 BEGIN
  SET @command = N'KILL ' + CONVERT(nvarchar(10), @session);
  EXEC (@command);
  FETCH NEXT FROM readers INTO @session;
END;
CLOSE readers;
DEALLOCATE readers;
""")
    print("Disconnected only the owned fixture's SQL receiver sessions")


def stop(directory):
    state = json.loads((directory / "fixture.json").read_text(encoding="utf-8"))
    if state.get("stopped"):
        print("Fixture is already stopped")
        return
    metadata = owned_fixture(directory)
    try:
        (directory / "server.log").write_text(
            run(["docker", "logs", metadata["container"]]), encoding="utf-8"
        )
    finally:
        # A diagnostic failure must not leave an otherwise removable owned server running.
        run(["docker", "rm", "--force", metadata["container"]])
        for filename in ["reader-password.txt", "setup.sql", "ca.key", "server.key",
                         "original.key", "replacement.key", "expired.key"]:
            path = directory / filename
            if path.exists():
                path.unlink()
        metadata["stopped"] = True
        (directory / "fixture.json").write_text(json.dumps(metadata, indent=2), encoding="utf-8")
    print("Removed fixture container and generated secrets")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("operation", choices=["start", "stop", "rotate-certificate", "disconnect-readers"])
    parser.add_argument("--directory", required=True, type=Path)
    parser.add_argument(
        "--allow-unencrypted", action="store_true",
        help="Allow non-TLS client connections to this disposable fixture; setup still verifies TLS.",
    )
    parser.add_argument("--certificate", choices=["valid", "replacement", "expired"])
    args = parser.parse_args()
    directory = args.directory.resolve()
    if args.certificate and args.operation != "rotate-certificate":
        parser.error("--certificate applies only to rotate-certificate")
    if args.operation == "start":
        start(directory, allow_unencrypted=args.allow_unencrypted)
    else:
        if args.allow_unencrypted:
            parser.error("--allow-unencrypted applies only to start")
        if args.operation == "rotate-certificate":
            if not args.certificate:
                parser.error("rotate-certificate requires --certificate")
            rotate_certificate(directory, args.certificate)
        elif args.operation == "disconnect-readers":
            disconnect_readers(directory)
        else:
            stop(directory)


if __name__ == "__main__":
    try:
        main()
    except (RuntimeError, subprocess.TimeoutExpired, OSError, ValueError) as exc:
        print(f"Fixture failed: {type(exc).__name__}: {exc}", file=sys.stderr)
        sys.exit(1)
