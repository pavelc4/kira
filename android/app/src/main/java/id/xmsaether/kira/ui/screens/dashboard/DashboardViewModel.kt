package id.xmsaether.kira.ui.screens.dashboard

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import id.xmsaether.kira.data.model.DeviceInfo
import id.xmsaether.kira.data.model.PerformanceProfile
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
    val error: String? = null
)

class DashboardViewModel : ViewModel() {
    private val _uiState = MutableStateFlow(DashboardUiState())
    val uiState: StateFlow<DashboardUiState> = _uiState.asStateFlow()

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
                    _uiState.value = _uiState.value.copy(performance = perf)
                } catch (e: Exception) {
                    // Lanjutkan polling meskipun error sesekali
                }
                delay(1000L) // Polling setiap 1 detik, sama seperti versi desktop
            }
        }
    }
}
