package com.sensefoundry.data.sync

internal class SyncTokenRegressionException(
    currentToken: Long,
    receivedToken: Long,
) : IllegalStateException(
    "sync token regressed from $currentToken to $receivedToken",
)

internal fun requireMonotonicSyncToken(currentToken: Long, receivedToken: Long): Long {
    if (receivedToken < currentToken) {
        throw SyncTokenRegressionException(currentToken, receivedToken)
    }
    return receivedToken
}
