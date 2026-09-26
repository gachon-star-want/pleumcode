package io.github.gachon_star_want

import com.sun.jna.Platform
import java.nio.file.Files

internal object NativeLibraryLoader {
    init {
        val componentName = "pleum"
        if (System.getProperty("uniffi.component.$componentName.libraryOverride") == null) {
            val resource = nativeResourcePath()
            val stream = NativeLibraryLoader::class.java.classLoader.getResourceAsStream(resource)
                ?: error("Pleum SDK native library resource not found: $resource")
            val library = Files.createTempFile("pleum-sdk-", nativeLibraryFileName()).toFile()
            library.deleteOnExit()
            stream.use { input -> library.outputStream().use { output -> input.copyTo(output) } }
            System.setProperty("uniffi.component.$componentName.libraryOverride", library.absolutePath)
        }
    }

    fun ensureLoaded() = Unit

    private fun nativeResourcePath(): String = "${jnaResourcePrefix()}/${nativeLibraryFileName()}"

    private fun nativeLibraryFileName(): String = when (osName()) {
        "darwin" -> "libpleum_sdk.dylib"
        "linux" -> "libpleum_sdk.so"
        "win32" -> "pleum_sdk.dll"
        else -> error("Unsupported OS: ${System.getProperty("os.name")}")
    }

    private fun jnaResourcePrefix(): String = "${osName()}-${archName()}"

    private fun osName(): String = when {
        System.getProperty("os.name").startsWith("Mac OS X") -> "darwin"
        System.getProperty("os.name").startsWith("Linux") -> "linux"
        System.getProperty("os.name").startsWith("Windows") -> "win32"
        else -> System.getProperty("os.name").lowercase().replace(Regex("\\s+"), "-")
    }

    private fun archName(): String = when {
        Platform.isARM() && Platform.is64Bit() -> "aarch64"
        Platform.isIntel() && Platform.is64Bit() -> "x86-64"
        else -> System.getProperty("os.arch")
    }
}
