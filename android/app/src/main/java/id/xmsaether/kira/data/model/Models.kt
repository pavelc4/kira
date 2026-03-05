package id.xmsaether.kira.data.model

data class DeviceInfo(
    val model: String = "",
    val manufacturer: String = "",
    val androidVersion: String = "",
    val abi: String = "",
    val slot: String = "",
    val battery: Int = 0,
    val screenResolution: String = "",
    val refreshRate: Int = 0,
    val securityPatch: String = "",
    val buildId: String = "",
    val storageTotalGb: String = "",
    val storageUsedGb: String = "",
    val storageFreeGb: String = ""
)

data class MemoryInfo(
    val totalKb: Long = 0,
    val freeKb: Long = 0,
    val availableKb: Long = 0
)


data class CpuTimes(
    val user: Long = 0,
    val nice: Long = 0,
    val sys: Long = 0,
    val idle: Long = 0,
    val iowait: Long = 0,
    val irq: Long = 0,
    val softirq: Long = 0
)


data class CpuInfo(
    val name: String = "",
    val times: CpuTimes = CpuTimes(),
    val speedMhz: Int? = null
)


data class BatteryInfo(
    val level: Int = 0,
    val temperature: Int = 0,
    val voltage: Int = 0
)


data class FpsData(
    val flips: Long = 0,
    val timestampMs: Long = 0,
    val isDirect: Boolean = false
)


data class PerformanceProfile(
    val memory: MemoryInfo? = null,
    val battery: BatteryInfo? = null,
    val cpuCores: List<CpuInfo> = emptyList(),
    val fps: FpsData? = null,
    val uptimeSeconds: Long = 0
)
