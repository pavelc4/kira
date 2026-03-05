#include <jni.h>
#include <string>

// This is a placeholder for the JNI methods that will wrap the Rust library.
// For now, this is just to make the CMake build pass.
extern "C" JNIEXPORT jstring JNICALL
Java_id_xmsaether_kira_jni_KiraCoreNative_stringFromJNI(
        JNIEnv* env,
        jobject /* this */) {
    std::string hello = "Hello from C++";
    return env->NewStringUTF(hello.c_str());
}
