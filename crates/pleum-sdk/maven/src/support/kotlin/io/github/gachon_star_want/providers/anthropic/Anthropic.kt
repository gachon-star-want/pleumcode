package io.github.gachon_star_want.providers.anthropic

public fun provider(
    apiKey: String,
    baseUrl: String? = null,
    betaHeaders: List<String> = emptyList(),
): io.github.gachon_star_want.Provider = io.github.gachon_star_want.anthropicProvider(apiKey, baseUrl, betaHeaders)

public fun defaultModel(): String = io.github.gachon_star_want.anthropicDefaultModel()
