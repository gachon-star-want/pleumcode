package io.github.gachon_star_want.providers.databricks

public fun provider(host: String, token: String): io.github.gachon_star_want.Provider =
    io.github.gachon_star_want.databricksProvider(host, token)

public fun defaultModel(): String = io.github.gachon_star_want.databricksDefaultModel()
