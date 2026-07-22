package com.sensefoundry.data.sync

import org.junit.Assert.assertEquals
import org.junit.Assert.assertThrows
import org.junit.Test

class SyncProtocolTest {
    @Test
    fun acceptsUnchangedToken() {
        assertEquals(7L, requireMonotonicSyncToken(7L, 7L))
    }

    @Test
    fun acceptsNewerToken() {
        assertEquals(8L, requireMonotonicSyncToken(7L, 8L))
    }

    @Test
    fun rejectsRegressedToken() {
        assertThrows(SyncTokenRegressionException::class.java) {
            requireMonotonicSyncToken(7L, 0L)
        }
    }
}
