package id.xmsaether.kira.data.repository

import id.xmsaether.kira.core.shell.ShellExecutor
import id.xmsaether.kira.data.model.*


object DeviceRepository {

    suspend fun getDeviceInfo(): DeviceInfo {
        val model = ShellExecutor.execute("getprop ro.product.model").stdout
        val manufacturer = ShellExecutor.execute("getprop ro.product.manufacturer").stdout
        val androidVersion = ShellExecutor.execute("getprop ro.build.version.release").stdout
        val abi = ShellExecutor.execute("getprop ro.product.cpu.abi").stdout
        val slot = ShellExecutor.execute("getprop ro.boot.slot_suffix").stdout
        val securityPatch = ShellExecutor.execute("getprop ro.build.version.security_patch").stdout
        val buildId = ShellExecutor.execute("getprop ro.build.display.id").stdout
        val screenResolution = ShellExecutor.execute("wm size").stdout
        val refreshRateStr = ShellExecutor.execute("settings get system peak_refresh_rate").stdout
        val refreshRate = refreshRateStr.toFloatOrNull()?.toInt() ?: 60

        return DeviceInfo(
            model = model,
            manufacturer = manufacturer,
            androidVersion = androidVersion,
            abi = abi,
            slot = slot,
            screenResolution = screenResolution,
            refreshRate = refreshRate,
            securityPatch = securityPatch,
            buildId = buildId
        )
    }

    suspend fun getPerformanceProfile(): PerformanceProfile {
        val memory = getMemoryInfo()
        val battery = getBatteryInfo()
        val cpuCores = getCpuInfo()
        val fps = getFps()
        val uptime = getUptime()

        return PerformanceProfile(
            memory = memory,
            battery = battery,
            cpuCores = cpuCores,
            fps = fps,
            uptimeSeconds = uptime
        )
    }

    private suspend fun getMemoryInfo(): MemoryInfo? {
        val output = ShellExecutor.execute("cat /proc/meminfo").stdout
        return parseMeminfo(output)
    }

    fun parseMeminfo(output: String): MemoryInfo? {
        var totalKb = 0L
        var freeKb = 0L
        var availableKb = 0L

        for (line in output.lines()) {
            val parts = line.trim().split("\\s+".toRegex())
            if (parts.size >= 2) {
                val value = parts[1].toLongOrNull() ?: continue
                when (parts[0]) {
                    "MemTotal:" -> totalKb = value
                    "MemFree:" -> freeKb = value
                    "MemAvailable:" -> availableKb = value
                }
            }
        }

        return if (totalKb > 0) MemoryInfo(totalKb, freeKb, availableKb) else null
    }

    private suspend fun getBatteryInfo(): BatteryInfo? {
        val output = ShellExecutor.execute("dumpsys battery").stdout
        return parseBatteryInfo(output)
    }

    fun parseBatteryInfo(output: String): BatteryInfo? {
        var level = 0
        var temperature = 0
        var voltage = 0
        var found = false

        for (line in output.lines()) {
            val parts = line.trim().split(":", limit = 2)
            if (parts.size == 2) {
                val key = parts[0].trim()
                val value = parts[1].trim().toIntOrNull() ?: continue
                when (key) {
                    "level" -> { level = value; found = true }
                    "temperature" -> temperature = value
                    "voltage" -> voltage = value
                }
            }
        }

        return if (found) BatteryInfo(level, temperature, voltage) else null
    }

    
    private suspend fun getCpuInfo(): List<CpuInfo> {
        val output = ShellExecutor.execute("cat /proc/stat").stdout
        val cpus = parseCpuStat(output).toMutableList()
        val speedsOutput = ShellExecutor.execute(
            "for i in /sys/devices/system/cpu/cpu[0-9]*; do echo -n \"\${i##*/} \"; cat \$i/cpufreq/scaling_cur_freq 2>/dev/null || echo \"OFF\"; done"
        ).stdout

        val speedMap = mutableMapOf<String, Int>()
        for (line in speedsOutput.lines()) {
            val parts = line.trim().split("\\s+".toRegex())
            if (parts.size == 2) {
                val speed = parts[1].toIntOrNull()
                if (speed != null) {
                    speedMap[parts[0]] = speed / 1000 // KHz -> MHz
                }
            }
        }

        return cpus.map { cpu ->
            cpu.copy(speedMhz = speedMap[cpu.name])
        }
    }

    fun parseCpuStat(output: String): List<CpuInfo> {
        val cpus = mutableListOf<CpuInfo>()
        for (line in output.lines()) {
            val trimmed = line.trim()
            if (!trimmed.startsWith("cpu")) continue

            val parts = trimmed.split("\\s+".toRegex())
            if (parts.size >= 8) {
                val times = CpuTimes(
                    user = parts[1].toLongOrNull() ?: 0,
                    nice = parts[2].toLongOrNull() ?: 0,
                    sys = parts[3].toLongOrNull() ?: 0,
                    idle = parts[4].toLongOrNull() ?: 0,
                    iowait = parts[5].toLongOrNull() ?: 0,
                    irq = parts[6].toLongOrNull() ?: 0,
                    softirq = parts[7].toLongOrNull() ?: 0
                )
                cpus.add(CpuInfo(name = parts[0], times = times))
            }
        }
        return cpus
    }

    private suspend fun getFps(): FpsData? {
        val timestampMs = System.currentTimeMillis()

        val primary = ShellExecutor.executeAsRoot("cat /sys/class/drm/sde-crtc-0/measured_fps").stdout
        val fpsPrimary = parseDrmFps(primary)
        if (fpsPrimary != null && fpsPrimary > 0) {
            return FpsData(flips = (fpsPrimary * 100).toLong(), timestampMs = timestampMs, isDirect = true)
        }

        val fallback = ShellExecutor.executeAsRoot("cat /sys/class/drm/card0/fbc/fps").stdout
        val fpsFallback = fallback.trim().toDoubleOrNull()
        if (fpsFallback != null && fpsFallback > 0) {
            return FpsData(flips = (fpsFallback * 100).toLong(), timestampMs = timestampMs, isDirect = true)
        }

        return null
    }

    fun parseDrmFps(output: String): Double? {
        val trimmed = output.trim()
        if (trimmed.isBlank()) return null

        val fpsIdx = trimmed.indexOf("fps:")
        if (fpsIdx >= 0) {
            val afterFps = trimmed.substring(fpsIdx + 4).trim()
            val number = afterFps.takeWhile { it.isDigit() || it == '.' }
            return number.toDoubleOrNull()
        }

        return trimmed.split("\\s+".toRegex()).firstOrNull()?.toDoubleOrNull()
    }

    private suspend fun getUptime(): Long {
        val output = ShellExecutor.execute("cat /proc/uptime").stdout
        val parts = output.split("\\s+".toRegex())
        return parts.firstOrNull()?.toDoubleOrNull()?.toLong() ?: 0
    }

    suspend fun executeShellCommand(command: String) = ShellExecutor.execute(command)
    suspend fun executeShellCommandAsRoot(command: String) = ShellExecutor.executeAsRoot(command)
}
