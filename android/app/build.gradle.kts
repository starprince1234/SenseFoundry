plugins { id("com.android.application"); id("org.jetbrains.kotlin.android"); id("org.jetbrains.kotlin.plugin.compose"); id("com.google.devtools.ksp") }
android { namespace = "com.sensefoundry"; compileSdk = 35
    defaultConfig { applicationId = "com.sensefoundry"; minSdk = 26; targetSdk = 35; versionCode = 1; versionName = "0.1" }
    buildFeatures { compose = true }
}
dependencies {
    implementation(platform("androidx.compose:compose-bom:2025.05.00")); implementation("androidx.activity:activity-compose:1.10.1"); implementation("androidx.compose.material3:material3"); implementation("androidx.lifecycle:lifecycle-runtime-compose:2.9.0")
    implementation("androidx.room:room-runtime:2.7.1"); implementation("androidx.room:room-ktx:2.7.1"); ksp("androidx.room:room-compiler:2.7.1")
    implementation("androidx.work:work-runtime-ktx:2.10.1"); implementation("com.squareup.okhttp3:okhttp:4.12.0"); implementation("org.jetbrains.kotlinx:kotlinx-serialization-json:1.8.1")
}
