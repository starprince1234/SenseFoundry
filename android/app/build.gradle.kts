plugins { id("com.android.application"); id("org.jetbrains.kotlin.android"); id("org.jetbrains.kotlin.plugin.compose"); id("com.google.devtools.ksp") }

fun requiredEnvironment(name: String): String {
    val value = System.getenv(name)
    if (value.isNullOrBlank() || value.contains("your_value_here") || value.contains("placeholder")) {
        throw GradleException("$name must be injected by Doppler")
    }
    return value
}

fun buildConfigString(value: String): String = "\"" + value
    .replace("\\", "\\\\")
    .replace("\"", "\\\"")
    .replace("\r", "")
    .replace("\n", "\\n") + "\""

android { namespace = "com.sensefoundry"; compileSdk = 35
    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }
    defaultConfig {
        applicationId = "com.sensefoundry"
        minSdk = 26
        targetSdk = 35
        versionCode = 1
        versionName = "0.1"
        buildConfigField("String", "API_BASE_URL", "\"http://192.168.137.1:8080/api/v1\"")
        buildConfigField("String", "SYNC_SIGNING_PUBLIC_KEY", buildConfigString(requiredEnvironment("SYNC_SIGNING_PUBLIC_KEY")))
    }
    buildFeatures { compose = true; buildConfig = true }
}
kotlin { jvmToolchain(17) }
dependencies {
    implementation(platform("androidx.compose:compose-bom:2025.05.00")); implementation("androidx.activity:activity-compose:1.10.1"); implementation("androidx.compose.material3:material3"); implementation("androidx.lifecycle:lifecycle-runtime-compose:2.9.0")
    implementation("androidx.room:room-runtime:2.7.1"); implementation("androidx.room:room-ktx:2.7.1"); ksp("androidx.room:room-compiler:2.7.1")
    implementation("androidx.work:work-runtime-ktx:2.10.1"); implementation("com.squareup.okhttp3:okhttp:4.12.0"); implementation("org.jetbrains.kotlinx:kotlinx-serialization-json:1.8.1"); implementation("androidx.room:room-paging:2.7.1")
}
ksp { arg("room.schemaLocation", "$projectDir/schemas") }
