package com.example.pelancamic

interface Platform {
    val name: String
}

expect fun getPlatform(): Platform
