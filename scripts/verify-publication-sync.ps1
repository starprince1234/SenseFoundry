$ErrorActionPreference = 'Stop'

$apiBaseUrl = if ([string]::IsNullOrWhiteSpace($env:PUBLICATION_VERIFY_API_URL)) {
    'http://127.0.0.1:8080/api/v1'
} else {
    $env:PUBLICATION_VERIFY_API_URL.TrimEnd('/')
}

function Invoke-JsonPost([string]$Path, [object]$Body) {
    Invoke-RestMethod `
        -Method Post `
        -Uri "$apiBaseUrl$Path" `
        -ContentType 'application/json; charset=utf-8' `
        -Body ($Body | ConvertTo-Json -Depth 10 -Compress)
}

$publisherId = [guid]::NewGuid().ToString()
$firstReviewerId = [guid]::NewGuid().ToString()
$secondReviewerId = [guid]::NewGuid().ToString()
$headword = [string][char]0x6253
$definition = -join @(
    [char]0x7528, [char]0x624B, [char]0x6216, [char]0x5668,
    [char]0x5177, [char]0x649E, [char]0x51FB, [char]0x7269,
    [char]0x4F53, [char]0x3002
)
$sentence = -join @(
    [char]0x4ED6, [char]0x6B63, [char]0x5728, [char]0x6253,
    [char]0x9F13, [char]0x3002
)
$content = [ordered]@{
    senses = @(
        [ordered]@{
            id = 'sense-da-verb-hit'
            pos = 'verb'
            definition = $definition
        }
    )
    examples = @(
        [ordered]@{
            id = 'example-da-drum'
            sense_id = 'sense-da-verb-hit'
            sentence = $sentence
            rank = 1.0
        }
    )
}

$preview = Invoke-JsonPost '/publication-preview' ([ordered]@{
    headword = $headword
    content = $content
})
if ($preview.content_hash -notmatch '^[0-9a-f]{64}$') {
    throw 'Publication preview did not return a SHA-256 hash.'
}

$reviewTask = Invoke-JsonPost '/review-tasks' ([ordered]@{
    sense_candidate_id = [guid]::NewGuid().ToString()
    reviewer_ids = @($firstReviewerId, $secondReviewerId)
    reviewed_content_hash = $preview.content_hash
})

$publishRequest = [ordered]@{
    headword = $headword
    content = $content
    publisher_id = $publisherId
    review_task_id = $reviewTask.id
}

try {
    Invoke-JsonPost '/publications' $publishRequest | Out-Null
    throw 'Publication unexpectedly passed before two approvals.'
} catch {
    if ($_.Exception.Response.StatusCode.value__ -ne 409) {
        throw
    }
    Write-Output 'review_gate=blocked_before_approval'
}

Invoke-JsonPost "/review-tasks/$($reviewTask.id)/decide" ([ordered]@{
    reviewer_id = $firstReviewerId
    decision = 'approve'
    arbiter_id = $null
}) | Out-Null
$approvedTask = Invoke-JsonPost "/review-tasks/$($reviewTask.id)/decide" ([ordered]@{
    reviewer_id = $secondReviewerId
    decision = 'approve'
    arbiter_id = $null
})
if (-not $approvedTask.publishable -or $approvedTask.state -ne 'COMPLETED') {
    throw 'Review task did not become publishable after two approvals.'
}

$edition = Invoke-JsonPost '/publications' $publishRequest
$manifest = Invoke-RestMethod -Uri "$apiBaseUrl/sync-manifests/latest/delta?last_sync_token=0"
$manifestEdition = @($manifest.changed_editions | Where-Object { $_.edition_id -eq $edition.id })
if ($manifestEdition.Count -ne 1) {
    throw 'Published edition is missing from the latest sync manifest.'
}
if ($manifest.sync_token -lt 1) {
    throw 'Sync token was not advanced.'
}

$webClient = New-Object System.Net.WebClient
try {
    $deltaBytes = $webClient.DownloadData("$apiBaseUrl/editions/$($edition.id)/delta")
} finally {
    $webClient.Dispose()
}
$delta = [System.Text.Encoding]::UTF8.GetString($deltaBytes) | ConvertFrom-Json
if ($delta.headword -ne $headword -or $delta.version_number -lt 1) {
    throw 'Delta package is missing its headword or version.'
}
if (@($delta.senses).Count -ne 1 -or @($delta.examples).Count -ne 1) {
    throw 'Delta package does not contain the reviewed senses and examples.'
}

$env:SYNC_VERIFY_DELTA_BASE64 = [Convert]::ToBase64String($deltaBytes)
$env:SYNC_VERIFY_CONTENT_HASH = $manifestEdition[0].content_hash
$env:SYNC_VERIFY_SIGNATURE = $manifestEdition[0].signature
try {
    python scripts/verify-sync-signature.py
    if ($LASTEXITCODE -ne 0) {
        throw 'Independent signature verification failed.'
    }
} finally {
    Remove-Item Env:SYNC_VERIFY_DELTA_BASE64 -ErrorAction SilentlyContinue
    Remove-Item Env:SYNC_VERIFY_CONTENT_HASH -ErrorAction SilentlyContinue
    Remove-Item Env:SYNC_VERIFY_SIGNATURE -ErrorAction SilentlyContinue
}

Write-Output 'review_gate=two_distinct_approvals'
Write-Output 'publication=created'
Write-Output 'sync_manifest=contains_published_edition'
Write-Output 'delta_contract=headword_version_senses_examples'
