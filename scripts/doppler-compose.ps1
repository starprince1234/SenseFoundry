param(
    [Parameter(ValueFromRemainingArguments = $true)]
    [string[]]$ComposeArgs
)

$ErrorActionPreference = 'Stop'

function Require-EnvironmentVariable([string]$Name) {
    $value = [Environment]::GetEnvironmentVariable($Name)
    if ([string]::IsNullOrWhiteSpace($value) -or
        $value -match '(?i)changeme|your_value_here|placeholder') {
        throw "Required Doppler variable is missing or still a placeholder: $Name"
    }
    return $value
}

if (-not $ComposeArgs -or $ComposeArgs.Count -eq 0) {
    $ComposeArgs = @('config', '--quiet')
}

$operation = $ComposeArgs[0]
$buildOnlyOperations = @('build', 'images', 'pull', 'push', 'version')
if ($operation -notin $buildOnlyOperations) {
    $databaseUrl = Require-EnvironmentVariable 'DATABASE_URL'
    $databaseUri = [Uri]$databaseUrl
    $databaseCredentials = $databaseUri.UserInfo -split ':', 2
    if ($databaseCredentials.Count -ne 2) {
        throw 'DATABASE_URL must contain a username and password.'
    }

    if ([string]::IsNullOrWhiteSpace($env:POSTGRES_PASSWORD) -or
        $env:POSTGRES_PASSWORD -match '(?i)changeme|your_value_here|placeholder') {
        $env:POSTGRES_PASSWORD = [Uri]::UnescapeDataString($databaseCredentials[1])
    }

    $containerDatabaseUri = [UriBuilder]$databaseUri
    $containerDatabaseUri.Host = 'postgres'
    $env:DATABASE_URL = $containerDatabaseUri.Uri.AbsoluteUri

    Require-EnvironmentVariable 'KEYCLOAK_ADMIN_PASSWORD' | Out-Null
    Require-EnvironmentVariable 'LLM_API_URL' | Out-Null
    Require-EnvironmentVariable 'LLM_API_KEY' | Out-Null
    Require-EnvironmentVariable 'MINIO_ACCESS_KEY' | Out-Null
    Require-EnvironmentVariable 'MINIO_SECRET_KEY' | Out-Null
    Require-EnvironmentVariable 'OPENSEARCH_PASSWORD' | Out-Null
    Require-EnvironmentVariable 'KEYCLOAK_CLIENT_SECRET' | Out-Null
    Require-EnvironmentVariable 'SYNC_SIGNING_PRIVATE_KEY' | Out-Null
}

& docker compose @ComposeArgs
if ($LASTEXITCODE -ne 0) {
    exit $LASTEXITCODE
}
