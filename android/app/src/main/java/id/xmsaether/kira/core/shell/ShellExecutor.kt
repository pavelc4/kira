package id.xmsaether.kira.core.shell

import com.topjohnwu.superuser.Shell
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext

data class CommandOutput(
    val stdout: String,
    val stderr: String,
    val exitCode: Int,
    val durationMs: Long
)

object ShellExecutor {

    init {
        Shell.enableVerboseLogging = false
        Shell.setDefaultBuilder(
            Shell.Builder.create()
                .setFlags(Shell.FLAG_REDIRECT_STDERR or Shell.FLAG_MOUNT_MASTER)
                .setTimeout(10)
        )
    }

    suspend fun execute(command: String): CommandOutput = withContext(Dispatchers.IO) {
        val startTime = System.currentTimeMillis()
        try {
            val result = Shell.cmd(command).exec()
            CommandOutput(
                stdout = result.out.joinToString("\n").trim(),
                stderr = result.err.joinToString("\n").trim(),
                exitCode = result.code,
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
            val result = Shell.cmd(command).exec()
            CommandOutput(
                stdout = result.out.joinToString("\n").trim(),
                stderr = result.err.joinToString("\n").trim(),
                exitCode = result.code,
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

    fun isRootAvailable(): Boolean {
        return Shell.isAppGrantedRoot() == true
    }
}
