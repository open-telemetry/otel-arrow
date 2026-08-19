# Copyright The OpenTelemetry Authors
# SPDX-License-Identifier: Apache-2.0

[CmdletBinding()]
param(
    [string]$OracleClientDir = $env:ORACLE_CLIENT_LIB_DIR,
    [string]$ContainerName = "oracle-free-1",
    [string]$Query = "SELECT SYSDATE AS CURRENT_TIME FROM DUAL",
    [ValidateRange(1, 65535)]
    [int]$MaxRows = 10
)

$ErrorActionPreference = "Stop"
$passwordWasPrompted = $false

function Test-OciOnPath {
    foreach ($directory in $env:PATH -split ";") {
        if ($directory -and (Test-Path -LiteralPath (Join-Path $directory "oci.dll"))) {
            return $true
        }
    }
    return $false
}

try {
    if (-not (Get-Command docker -ErrorAction SilentlyContinue)) {
        throw "Docker is not installed or is not on PATH."
    }

    if ($OracleClientDir) {
        $ociDll = Join-Path $OracleClientDir "oci.dll"
        if (-not (Test-Path -LiteralPath $ociDll -PathType Leaf)) {
            throw "OracleClientDir does not contain oci.dll: $OracleClientDir"
        }
        $env:PATH = "$OracleClientDir;$env:PATH"
    }

    if (-not (Test-OciOnPath)) {
        throw "Oracle Instant Client is not on PATH. Pass -OracleClientDir or set ORACLE_CLIENT_LIB_DIR."
    }

    if (-not $env:ORACLE_PWD) {
        $securePassword = Read-Host "Local Oracle password" -AsSecureString
        $passwordPointer = [Runtime.InteropServices.Marshal]::SecureStringToBSTR($securePassword)
        try {
            $env:ORACLE_PWD = [Runtime.InteropServices.Marshal]::PtrToStringBSTR($passwordPointer)
            $passwordWasPrompted = $true
        }
        finally {
            [Runtime.InteropServices.Marshal]::ZeroFreeBSTR($passwordPointer)
        }
    }
    if (-not $env:ORACLE_PWD) {
        throw "The local Oracle password must not be empty."
    }

    $existingContainer = docker ps -a --filter "name=^${ContainerName}$" --format "{{.Names}}"
    if ($LASTEXITCODE -ne 0) {
        throw "Unable to query Docker containers."
    }
    if (-not $existingContainer) {
        Write-Host "Creating Oracle Free container $ContainerName..."
        docker run --detach `
            --name $ContainerName `
            --env ORACLE_PWD `
            --publish 1521:1521 `
            container-registry.oracle.com/database/free:latest-lite | Out-Null
        if ($LASTEXITCODE -ne 0) {
            throw "Unable to create Oracle Free container $ContainerName."
        }
    }
    else {
        $running = docker inspect --format "{{.State.Running}}" $ContainerName
        if ($LASTEXITCODE -ne 0) {
            throw "Unable to inspect Oracle Free container $ContainerName."
        }
        if ($running -ne "true") {
            Write-Host "Starting Oracle Free container $ContainerName..."
            docker start $ContainerName | Out-Null
            if ($LASTEXITCODE -ne 0) {
                throw "Unable to start Oracle Free container $ContainerName."
            }
        }
    }

    Write-Host "Waiting for Oracle Free to become healthy..."
    $healthy = $false
    for ($attempt = 0; $attempt -lt 120; $attempt++) {
        $status = docker inspect --format "{{.State.Status}} {{if .State.Health}}{{.State.Health.Status}}{{end}}" $ContainerName
        if ($LASTEXITCODE -ne 0) {
            throw "Unable to inspect Oracle Free health."
        }
        if ($status -eq "running healthy") {
            $healthy = $true
            break
        }
        if ($status.StartsWith("exited")) {
            docker logs --tail 100 $ContainerName
            throw "Oracle Free container exited during startup."
        }
        Start-Sleep -Seconds 2
    }
    if (-not $healthy) {
        throw "Oracle Free did not become healthy within four minutes."
    }

    if (-not $env:ORACLE_USERNAME) {
        $env:ORACLE_USERNAME = "PDBADMIN"
    }
    if (-not $env:ORACLE_CONNECT_STRING) {
        $env:ORACLE_CONNECT_STRING = "//localhost:1521/FREEPDB1"
    }

    $workspace = (Resolve-Path (Join-Path $PSScriptRoot "..\..")).Path
    Push-Location $workspace
    try {
        cargo run -p otap-df-oracle-oci-smoke -- --local-free $Query $MaxRows
        if ($LASTEXITCODE -ne 0) {
            throw "Oracle OCI smoke test failed with exit code $LASTEXITCODE."
        }
    }
    finally {
        Pop-Location
    }
}
finally {
    if ($passwordWasPrompted) {
        Remove-Item Env:ORACLE_PWD
    }
}
