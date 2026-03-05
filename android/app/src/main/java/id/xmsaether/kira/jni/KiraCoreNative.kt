package id.xmsaether.kira.jni

object KiraCoreNative {
    // Load the native library compiled from Rust
    init {
        try {
            System.loadLibrary("kiracore_jni")
        } catch (e: UnsatisfiedLinkError) {
            e.printStackTrace()
        }
    }

    /**
     * A temporary test string from Rust/C++
     */
    external fun stringFromJNI(): String

    // Example methods mapping to the desktop `KiraCore.rs`
    
    // external fun getDeviceInfo(): String // Could return JSON
    // external fun rebootDevice(mode: String): Boolean
    // external fun listPackages(filter: String): String // Could return JSON
    // external fun listProcesses(): String // Could return JSON
    // external fun getPerformanceProfile(): String // Could return JSON
    // external fun executeShellCommand(command: String): String
}
