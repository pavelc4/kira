package id.xmsaether.kira.core.shizuku

import android.content.pm.PackageManager
import android.os.Build
import rikka.shizuku.Shizuku
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow

object ShizukuHelper {
    private val _isReady = MutableStateFlow(false)
    val isReady: StateFlow<Boolean> = _isReady.asStateFlow()

    private val binderReceivedListener = Shizuku.OnBinderReceivedListener {
        _isReady.value = true
        checkPermission()
    }

    private val binderDeadListener = Shizuku.OnBinderDeadListener {
        _isReady.value = false
    }

    private val onRequestPermissionResultListener = Shizuku.OnRequestPermissionResultListener { requestCode, grantResult ->
        if (requestCode == 1 && grantResult == PackageManager.PERMISSION_GRANTED) {
             _isReady.value = true
        }
    }

    fun init() {
        Shizuku.addBinderReceivedListener(binderReceivedListener)
        Shizuku.addBinderDeadListener(binderDeadListener)
        Shizuku.addRequestPermissionResultListener(onRequestPermissionResultListener)
        _isReady.value = Shizuku.pingBinder()
        if (_isReady.value) {
            checkPermission()
        }
    }

    fun destroy() {
        Shizuku.removeBinderReceivedListener(binderReceivedListener)
        Shizuku.removeBinderDeadListener(binderDeadListener)
        Shizuku.removeRequestPermissionResultListener(onRequestPermissionResultListener)
    }

    private fun checkPermission(): Boolean {
        if (Shizuku.isPreV11()) return false
        
        return try {
            if (Shizuku.checkSelfPermission() == PackageManager.PERMISSION_GRANTED) {
                true
            } else if (Shizuku.shouldShowRequestPermissionRationale()) {
                false
            } else {
                Shizuku.requestPermission(1)
                false
            }
        } catch (e: Exception) {
            false
        }
    }
}
