package id.xmsaether.kira.ui.screens.dashboard

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import id.xmsaether.kira.data.model.*
import id.xmsaether.kira.data.repository.DeviceRepository
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.launch

data class DashboardUiState(
    val deviceInfo: DeviceInfo? = null,
    val performance: PerformanceProfile? = null,
    val isLoading: Boolean = true,
    val error: String? = null,
    val cpuHistory: List<Float> = List(40) { 0f },
    val memHistory: List<Float> = List(40) { 0f },
    val fpsHistory: List<Float> = List(40) { 0f },
    val coreHistories: Map<Int, List<Float>> = emptyMap(),
    val currentCpu: Int = 0,
    val currentMem: Int = 0,
    val currentFps: Int = 0,
    val memUsedMb: Int = 0,
    val memTotalMb: Int = 0,
    val coreUsages: Map<Int, Int> = emptyMap(),
    val coreSpeeds: Map<Int, Int> = emptyMap(),
    val uptimeStr: String = "0:00:00"
)

class DashboardViewModel : ViewModel() {
    private val _uiState = MutableStateFlow(DashboardUiState())
    val uiState: StateFlow<DashboardUiState> = _uiState.asStateFlow()

    private var lastCpuData: List<CpuInfo>? = null
    private var lastFpsData: FpsData? = null

    init {
        loadDeviceInfo()
        startPerformancePolling()
    }

    private fun loadDeviceInfo() {
        viewModelScope.launch {
            try {
                val info = DeviceRepository.getDeviceInfo()
                _uiState.value = _uiState.value.copy(deviceInfo = info, isLoading = false)
            } catch (e: Exception) {
                _uiState.value = _uiState.value.copy(error = e.message, isLoading = false)
            }
        }
    }

    private fun startPerformancePolling() {
        viewModelScope.launch {
            while (true) {
                try {
                    val perf = DeviceRepository.getPerformanceProfile()
                    val state = _uiState.value

                    var cpuPercent = state.currentCpu
                    val cpuHist = state.cpuHistory.toMutableList()
                    val coreHists = state.coreHistories.toMutableMap()
                    val coreUsages = state.coreUsages.toMutableMap()
                    val coreSpeeds = state.coreSpeeds.toMutableMap()

                    if (perf.cpuCores.isNotEmpty() && lastCpuData != null) {
                        val curr = perf.cpuCores
                        val prev = lastCpuData!!

                        if (curr.isNotEmpty() && prev.isNotEmpty()) {
                            val cTotal = curr[0].times.totalTicks()
                            val pTotal = prev[0].times.totalTicks()
                            val totalDelta = cTotal - pTotal
                            val idleDelta = (curr[0].times.idle + curr[0].times.iowait) -
                                    (prev[0].times.idle + prev[0].times.iowait)

                            if (totalDelta > 0) {
                                cpuPercent = (100 * (1 - idleDelta.toFloat() / totalDelta)).toInt().coerceIn(0, 100)
                            }

                            for (i in 1 until curr.size) {
                                val cpuName = curr[i].name
                                if (!cpuName.startsWith("cpu")) continue
                                val idx = cpuName.removePrefix("cpu").toIntOrNull() ?: continue
                                val prevCore = prev.find { it.name == cpuName } ?: continue

                                val coreTotDelta = curr[i].times.totalTicks() - prevCore.times.totalTicks()
                                val coreIdleDelta = (curr[i].times.idle + curr[i].times.iowait) -
                                        (prevCore.times.idle + prevCore.times.iowait)

                                val u = if (coreTotDelta > 0) {
                                    (100 * (1 - coreIdleDelta.toFloat() / coreTotDelta)).toInt().coerceIn(0, 100)
                                } else 0

                                coreUsages[idx] = u
                                coreSpeeds[idx] = curr[i].speedMhz ?: 0

                                val hist = (coreHists[idx] ?: List(25) { 0f }).toMutableList()
                                hist.add(u.toFloat())
                                if (hist.size > 25) hist.removeAt(0)
                                coreHists[idx] = hist
                            }
                        }
                    }
                    lastCpuData = perf.cpuCores

                    cpuHist.add(cpuPercent.toFloat())
                    if (cpuHist.size > 40) cpuHist.removeAt(0)

                    var memPercent = state.currentMem
                    var memUsedMb = state.memUsedMb
                    var memTotalMb = state.memTotalMb
                    val memHist = state.memHistory.toMutableList()
                    perf.memory?.let { mem ->
                        if (mem.totalKb > 0) {
                            memPercent = ((mem.totalKb - mem.availableKb).toFloat() / mem.totalKb * 100).toInt()
                            memUsedMb = ((mem.totalKb - mem.availableKb) / 1024).toInt()
                            memTotalMb = (mem.totalKb / 1024).toInt()
                        }
                    }
                    memHist.add(memPercent.toFloat())
                    if (memHist.size > 40) memHist.removeAt(0)

                    var fpsVal = state.currentFps
                    val fpsHist = state.fpsHistory.toMutableList()
                    perf.fps?.let { fps ->
                        if (fps.isDirect) {
                            fpsVal = (fps.flips / 100).toInt()
                        } else {
                            val prev = lastFpsData
                            if (prev != null && fps.flips > prev.flips) {
                                val deltaFlips = fps.flips - prev.flips
                                val deltaTime = fps.timestampMs - prev.timestampMs
                                if (deltaTime > 0) {
                                    fpsVal = ((deltaFlips * 1000) / deltaTime).toInt()
                                }
                            } else if (prev != null && fps.flips == prev.flips) {
                                fpsVal = 0
                            }
                        }
                        lastFpsData = fps
                    }
                    fpsHist.add(fpsVal.toFloat())
                    if (fpsHist.size > 40) fpsHist.removeAt(0)

                    val uptimeStr = formatUptime(perf.uptimeSeconds)

                    _uiState.value = state.copy(
                        performance = perf,
                        cpuHistory = cpuHist,
                        memHistory = memHist,
                        fpsHistory = fpsHist,
                        coreHistories = coreHists,
                        currentCpu = cpuPercent,
                        currentMem = memPercent,
                        currentFps = fpsVal,
                        memUsedMb = memUsedMb,
                        memTotalMb = memTotalMb,
                        coreUsages = coreUsages,
                        coreSpeeds = coreSpeeds,
                        uptimeStr = uptimeStr
                    )
                } catch (_: Exception) {
                }
                delay(1000L)
            }
        }
    }

    private fun formatUptime(seconds: Long): String {
        val days = seconds / 86400
        val hours = (seconds % 86400) / 3600
        val minutes = (seconds % 3600) / 60
        val secs = seconds % 60
        return if (days > 0) {
            "${days}d ${hours}:${minutes.toString().padStart(2, '0')}:${secs.toString().padStart(2, '0')}"
        } else {
            "${hours}:${minutes.toString().padStart(2, '0')}:${secs.toString().padStart(2, '0')}"
        }
    }
}

private fun CpuTimes.totalTicks(): Long =
    user + nice + sys + idle + iowait + irq + softirq
