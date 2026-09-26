package io.github.gachon_star_want.providers.openai

public fun provider(apiKey: String): io.github.gachon_star_want.Provider = io.github.gachon_star_want.openaiProvider(apiKey)

public fun defaultModel(): String = io.github.gachon_star_want.openaiDefaultModel()
