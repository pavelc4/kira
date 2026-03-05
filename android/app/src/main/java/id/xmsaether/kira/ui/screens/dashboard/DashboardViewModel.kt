package id.xmsaether.kira.ui.screens.dashboard

import androidx.lifecycle.ViewModel
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow

data class DashboardUiState(
    val deviceName: String = "Fetching...",
    val isShizukuReady: Boolean = false,
    val cpuUsage: Int = 0,
    val memUsage: Int = 0,
    val fps: Int = 0
)

class DashboardViewModel : ViewModel() {
    private val _uiState = MutableStateFlow(DashboardUiState())
    val uiState: StateFlow<DashboardUiState> = _uiState.asStateFlow()

    init {
        // Initialize real data gathering through native/JNI and Shizuku
    }
}
