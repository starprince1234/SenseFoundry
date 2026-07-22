package com.sensefoundry.data.db

import android.content.Context
import androidx.room.*

@Entity data class Edition(@PrimaryKey val id: String, val headword: String, val versionNumber: Int, val contentHash: String, val signature: String)
@Entity(indices = [Index("editionId"), Index("headword")]) data class Sense(@PrimaryKey val id: String, val editionId: String, val headword: String, val partOfSpeech: String, val definition: String)
@Entity(indices = [Index("senseId")]) data class Example(@PrimaryKey val id: String, val senseId: String, val sentence: String, val rank: Double)

data class OfflineStats(val editionCount: Int, val senseCount: Int, val exampleCount: Int)

@Dao
interface SenseDao {
    @Query("SELECT * FROM Sense WHERE headword LIKE '%' || :query || '%' ORDER BY headword") suspend fun search(query: String): List<Sense>
    @Query("SELECT * FROM Sense WHERE id = :id LIMIT 1") suspend fun getSense(id: String): Sense?
    @Query("SELECT * FROM Example WHERE senseId = :senseId ORDER BY rank DESC") suspend fun getExamples(senseId: String): List<Example>
    @Query("SELECT (SELECT COUNT(*) FROM Edition) AS editionCount, (SELECT COUNT(*) FROM Sense) AS senseCount, (SELECT COUNT(*) FROM Example) AS exampleCount") suspend fun offlineStats(): OfflineStats
    @Transaction suspend fun replaceEdition(edition: Edition, senses: List<Sense>, examples: List<Example>) { deleteExamples(edition.id); deleteSenses(edition.id); insertEdition(edition); insertSenses(senses); insertExamples(examples) }
    @Insert(onConflict = OnConflictStrategy.REPLACE) suspend fun insertEdition(edition: Edition)
    @Insert(onConflict = OnConflictStrategy.REPLACE) suspend fun insertSenses(senses: List<Sense>)
    @Insert(onConflict = OnConflictStrategy.REPLACE) suspend fun insertExamples(examples: List<Example>)
    @Query("DELETE FROM Example WHERE senseId IN (SELECT id FROM Sense WHERE editionId = :editionId)") suspend fun deleteExamples(editionId: String)
    @Query("DELETE FROM Sense WHERE editionId = :editionId") suspend fun deleteSenses(editionId: String)
}

@Database(entities = [Edition::class, Sense::class, Example::class], version = 1, exportSchema = true)
abstract class SenseDatabase : RoomDatabase() {
    abstract fun senseDao(): SenseDao
    companion object { @Volatile private var instance: SenseDatabase? = null
        fun get(context: Context): SenseDatabase = instance ?: synchronized(this) { instance ?: Room.databaseBuilder(context.applicationContext, SenseDatabase::class.java, "sensefoundry.db").build().also { instance = it } }
    }
}
