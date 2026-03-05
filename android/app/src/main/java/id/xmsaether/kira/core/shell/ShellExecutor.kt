package id.xmsaether.kira.core.shell

import id.xmsaether.kira.core.shizuku.ShizukuHelper
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import java.io.BufferedReader
import java.io.InputStreamReader

data class CommandOutput(
    val stdout: String,
    val stderr: String,
    val exitCode: Int,
    val durationMs: Long
)

object ShellExecutor {

    suspend fun execute(command: String): CommandOutput = withContext(Dispatchers.IO) {
        val startTime = System.currentTimeMillis()
        try {
            val process = Runtime.getRuntime().exec(arrayOf("sh", "-c", command))
            
            val stdout = BufferedReader(InputStreamReader(process.inputStream)).use { it.readText() }
            val stderr = BufferedReader(InputStreamReader(process.errorStream)).use { it.readText() }
            val exitCode = process.waitFor()
            
            CommandOutput(
                stdout = stdout.trim(),
                stderr = stderr.trim(),
                exitCode = exitCode,
                durationMs = System.currentTimeMillis() - startTime
            )
        } catch (e: Exception) {
            CommandOutput(
                stdout = "",
                stderr = e.message ?: "Unknown error",
                exitCode = -1,
                durationMs = System.currentTimeMillis() - startTime
            )
        }
    }

    suspend fun executeAsRoot(command: String): CommandOutput = withContext(Dispatchers.IO) {
        val startTime = System.currentTimeMillis()
        try {
            val process = Runtime.getRuntime().exec(arrayOf("su", "-c", command))
            
            val stdout = BufferedReader(InputStreamReader(process.inputStream)).use { it.readText() }
            val stderr = BufferedReader(InputStreamReader(process.errorStream)).use { it.readText() }
            val exitCode = process.waitFor()
            
            CommandOutput(
                stdout = stdout.trim(),
                stderr = stderr.trim(),
                exitCode = exitCode,
                durationMs = System.currentTimeMillis() - startTime
            )
        } catch (e: Exception) {
            CommandOutput(
                stdout = "",
                stderr = e.message ?: "Unknown error",
                exitCode = -1,
                durationMs = System.currentTimeMillis() - startTime
            )
        }
    }
}
