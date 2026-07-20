$ErrorActionPreference = 'Stop'

function Require-EnvironmentVariable([string]$Name) {
    $value = [Environment]::GetEnvironmentVariable($Name)
    if ([string]::IsNullOrWhiteSpace($value) -or
        $value -match '(?i)changeme|your_value_here|placeholder') {
        throw "Doppler variable is missing or still a placeholder: $Name"
    }
    return $value
}

$embeddingApiUrl = (Require-EnvironmentVariable 'EMBEDDING_API_URL').TrimEnd('/')
$embeddingApiKey = Require-EnvironmentVariable 'EMBEDDING_API_KEY'
$embeddingModel = Require-EnvironmentVariable 'EMBEDDING_MODEL'
$chineseSentence = [string]::Concat(
    [char]0x4ED6,
    [char]0x6253,
    [char]0x5F00,
    [char]0x4E86,
    [char]0x95E8,
    [char]0x3002
)
$embeddingPath = ([Uri]$embeddingApiUrl).AbsolutePath
$embeddingEndpoint = if ($embeddingApiUrl.EndsWith('/embeddings')) {
    $embeddingApiUrl
} elseif ($embeddingPath -eq '/' -or [string]::IsNullOrWhiteSpace($embeddingPath)) {
    "$embeddingApiUrl/v1/embeddings"
} else {
    "$embeddingApiUrl/embeddings"
}

$embeddingBody = @{
    model = $embeddingModel
    input = @($chineseSentence)
} | ConvertTo-Json -Depth 4
$embeddingResponse = Invoke-RestMethod `
    -Method Post `
    -Uri $embeddingEndpoint `
    -Headers @{ Authorization = "Bearer $embeddingApiKey" } `
    -ContentType 'application/json; charset=utf-8' `
    -Body $embeddingBody `
    -TimeoutSec 60
if ($null -eq $embeddingResponse.data) {
    $responseType = $embeddingResponse.GetType().FullName
    $propertyNames = $embeddingResponse.PSObject.Properties.Name -join ','
    throw "Embedding response is not OpenAI-compatible. Type=$responseType Properties=$propertyNames"
}
$vector = $embeddingResponse.data[0].embedding
if ($null -eq $vector -or $vector.Count -le 0) {
    throw 'Embedding API returned no vector.'
}
Write-Output "EMBEDDING_STATUS=ok"
Write-Output "EMBEDDING_MODEL=$embeddingModel"
Write-Output "EMBEDDING_DIMENSION=$($vector.Count)"

$llmApiUrl = (Require-EnvironmentVariable 'LLM_API_URL').TrimEnd('/')
$llmPath = ([Uri]$llmApiUrl).AbsolutePath
if ($llmPath -eq '/' -or [string]::IsNullOrWhiteSpace($llmPath)) {
    $llmApiUrl = "$llmApiUrl/v1/chat/completions"
}
$llmApiKey = Require-EnvironmentVariable 'LLM_API_KEY'
$llmModel = Require-EnvironmentVariable 'LLM_MODEL'
$headword = [string]::Concat([char]0x6253, [char]0x5F00)
$llmBody = @{
    model = $llmModel
    temperature = 0
    messages = @(
        @{
            role = 'user'
            content = "Based only on evidence E1 ($chineseSentence), draft one candidate definition for $headword. Do not add sources, dates, authors, or examples. Return only the definition."
        }
    )
} | ConvertTo-Json -Depth 6
$llmResponse = Invoke-RestMethod `
    -Method Post `
    -Uri $llmApiUrl `
    -Headers @{ Authorization = "Bearer $llmApiKey" } `
    -ContentType 'application/json; charset=utf-8' `
    -Body $llmBody `
    -TimeoutSec 60
$definition = $llmResponse.choices[0].message.content
if ([string]::IsNullOrWhiteSpace($definition)) {
    throw 'LLM API returned no completion content.'
}
Write-Output "LLM_STATUS=ok"
Write-Output "LLM_MODEL=$llmModel"
Write-Output "LLM_RESPONSE_CHARS=$($definition.Length)"
