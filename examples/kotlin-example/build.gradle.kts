plugins {
    kotlin("jvm") version "1.6.10"
    id("java")
    idea
}

repositories {
    mavenLocal()
    mavenCentral()
}

tasks.withType<org.jetbrains.kotlin.gradle.tasks.KotlinCompile> {
    sourceCompatibility = "11"
    targetCompatibility = "11"

    kotlinOptions {
        freeCompilerArgs = listOf(
            "-Xjsr305=strict",
            "-Xopt-in=kotlinx.coroutines.ExperimentalCoroutinesApi"
        )
        jvmTarget = "11"
    }
}

dependencies {
    implementation("org.jetbrains.kotlin:kotlin-stdlib-jdk8:1.6.10")
    implementation("org.bouncycastle:bcprov-jdk15on:1.70")
    implementation("com.fasterxml.jackson.datatype:jackson-datatype-jdk8:2.13.3")
    implementation("com.fasterxml.jackson.datatype:jackson-datatype-jsr310:2.13.3")
    implementation("com.fasterxml.jackson.module:jackson-module-kotlin:2.13.3")
    implementation("com.google.protobuf:protobuf-java:3.21.1")
    implementation("com.google.protobuf:protobuf-java-util:3.21.1")
    implementation("io.provenance.client:pb-grpc-client-kotlin:1.1.1")
    implementation("io.provenance:proto-kotlin:1.10.0")
    implementation("io.provenance.scope:encryption:0.6.0")
    implementation("io.provenance.scope:util:0.6.0")
}
