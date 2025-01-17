import org.jetbrains.kotlin.gradle.tasks.KotlinCompile

plugins {
    kotlin("jvm") version "1.8.0"
    application
}

group = "org.example"
version = "1.0-SNAPSHOT"

repositories {
    mavenCentral()
}

dependencies {
    implementation(kotlin("stdlib-jdk8"))

    implementation("com.codeborne:selenide-appium:2.6.1")
    implementation("com.squareup.moshi:moshi-kotlin:1.14.0")
    implementation("io.appium:java-client:8.3.0")
    implementation("io.qameta.allure:allure-selenide:2.21.0")
    implementation("io.github.ashwithpoojary98:appium_flutterfinder_java:1.0.1")
    implementation("io.rest-assured:rest-assured:5.3.0")
    implementation("org.junit.jupiter:junit-jupiter:5.9.2")
    implementation("org.junit-pioneer:junit-pioneer:2.0.0")
    implementation("org.junit.platform:junit-platform-suite-engine:1.9.3")
    implementation("org.slf4j:slf4j-simple:2.0.6")
}

tasks.test {
    useJUnitPlatform()
}

kotlin {
    jvmToolchain(11)
}

application {
    mainClass.set("MainKt")
}
val compileKotlin: KotlinCompile by tasks
compileKotlin.kotlinOptions {
    jvmTarget = "11"
}
val compileTestKotlin: KotlinCompile by tasks
compileTestKotlin.kotlinOptions {
    jvmTarget = "11"
}
