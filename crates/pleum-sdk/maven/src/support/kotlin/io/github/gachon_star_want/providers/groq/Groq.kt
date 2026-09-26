package io.github.gachon_star_want.providers.groq

public fun provider(apiKey: String): io.github.gachon_star_want.Provider = io.github.gachon_star_want.groqProvider(apiKey)

public fun defaultModel(): String = io.github.gachon_star_want.groqDefaultModel()
