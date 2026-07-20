$ErrorActionPreference = 'Stop'

$required = @(
    'DATABASE_URL',
    'EMBEDDING_API_URL',
    'EMBEDDING_API_KEY',
    'EMBEDDING_MODEL',
    'LLM_API_URL',
    'LLM_API_KEY',
    'LLM_MODEL',
    'OPENSEARCH_PASSWORD',
    'MINIO_ACCESS_KEY',
    'MINIO_SECRET_KEY',
    'KEYCLOAK_CLIENT_SECRET',
    'SYNC_SIGNING_PRIVATE_KEY',
    'SYNC_SIGNING_PUBLIC_KEY'
)

$invalid = @()
foreach ($name in $required) {
    $value = [Environment]::GetEnvironmentVariable($name)
    $configured = -not [string]::IsNullOrWhiteSpace($value) -and
        $value -notmatch '(?i)changeme|your_value_here|placeholder'
    if (-not $configured) {
        $invalid += $name
    }
    Write-Output ("{0}={1}" -f $name, $(if ($configured) { 'configured' } else { 'missing_or_placeholder' }))
}

if ($invalid.Count -gt 0) {
    throw "Doppler validation failed for: $($invalid -join ', ')"
}

$databaseUri = [Uri]$env:DATABASE_URL
$embeddingUri = [Uri]$env:EMBEDDING_API_URL
$llmUri = [Uri]$env:LLM_API_URL
Write-Output "DATABASE_URL_HOST=$($databaseUri.Host)"
Write-Output "DATABASE_URL_DATABASE=$($databaseUri.AbsolutePath.TrimStart('/'))"
Write-Output "EMBEDDING_API_ORIGIN=$($embeddingUri.Scheme)://$($embeddingUri.Authority)"
Write-Output "EMBEDDING_API_PATH=$($embeddingUri.AbsolutePath)"
Write-Output "LLM_API_ORIGIN=$($llmUri.Scheme)://$($llmUri.Authority)"
Write-Output "LLM_API_PATH=$($llmUri.AbsolutePath)"
