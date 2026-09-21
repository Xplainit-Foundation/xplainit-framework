/*
 * Xplainit JVMTI Agent
 * ====================
 *
 * A native JVMTI agent that captures MethodEntry / MethodExit / Exception
 * events from a running JVM and forwards them to the Xplainit runtime.
 *
 * Wiring: the agent does NOT record directly. Instead it calls the static
 * recording methods on io.xplainit.Xplainit (nativeAgent* -> process-wide Rust
 * runtime in libxplainit_java.so) via JNI. This keeps a single source of truth
 * for the event store and reuses the tested Rust recording path.
 *
 * Filtering: recording every JDK/bootstrap method would be enormous and slow,
 * so callbacks are gated to classes whose signature starts with a configured
 * package prefix (default "Lio/xplainit/examples/"). This mirrors the Python
 * tracer's stdlib-exclusion idea. Override with -agentpath:...=<pkgPrefix>
 * where <pkgPrefix> uses '/' separators, e.g. "com/acme/app".
 *
 * Build:
 *   gcc -shared -fPIC -o libxplainit_agent.so native/agent.c \
 *       -I$JAVA_HOME/include -I$JAVA_HOME/include/linux
 *
 * Run:
 *   java -agentpath:$PWD/libxplainit_agent.so -cp <classes> \
 *        io.xplainit.examples.BasicExample
 */

#include <jvmti.h>
#include <jni.h>
#include <string.h>
#include <stdlib.h>
#include <stdio.h>

/* Default class-signature prefix we trace. JVMTI class signatures look like
 * "Lio/xplainit/examples/BasicExample;". */
#define DEFAULT_PREFIX "Lio/xplainit/examples/"

static char g_prefix[512] = DEFAULT_PREFIX;

/* Cached references to the Java recording bridge. Resolved in VMInit once the
 * class loader is ready. */
static jclass    g_xplainit_class      = NULL;
static jmethodID g_mid_init            = NULL;
static jmethodID g_mid_on_enter        = NULL;
static jmethodID g_mid_on_exit         = NULL;
static jmethodID g_mid_on_exception    = NULL;
static int       g_bridge_ready        = 0;

/* Guard so we never re-enter our own bridge (calling Java from a MethodEntry
 * callback would otherwise fire MethodEntry again for the bridge methods). */
static __thread int g_in_callback = 0;

/* Return 1 if the class signature is one we want to trace. */
static int should_trace(const char *class_sig) {
    if (class_sig == NULL) {
        return 0;
    }
    size_t plen = strlen(g_prefix);
    return strncmp(class_sig, g_prefix, plen) == 0;
}

/* Fetch the declaring class signature for a method. Caller frees via Deallocate.
 * Returns a malloc-free JVMTI-allocated string in *out_sig (may be NULL). */
static void get_class_signature(jvmtiEnv *jvmti, jmethodID method, char **out_sig) {
    *out_sig = NULL;
    jclass declaring = NULL;
    if ((*jvmti)->GetMethodDeclaringClass(jvmti, method, &declaring) != JVMTI_ERROR_NONE) {
        return;
    }
    char *sig = NULL;
    if ((*jvmti)->GetClassSignature(jvmti, declaring, &sig, NULL) == JVMTI_ERROR_NONE) {
        *out_sig = sig;
    }
}

/* ===== JVMTI event callbacks ===== */

static void JNICALL
cb_method_entry(jvmtiEnv *jvmti, JNIEnv *jni, jthread thread, jmethodID method) {
    (void)thread;
    if (!g_bridge_ready || g_in_callback) {
        return;
    }

    char *class_sig = NULL;
    get_class_signature(jvmti, method, &class_sig);
    if (!should_trace(class_sig)) {
        if (class_sig) (*jvmti)->Deallocate(jvmti, (unsigned char *)class_sig);
        return;
    }

    char *name = NULL;
    char *msig = NULL;
    (*jvmti)->GetMethodName(jvmti, method, &name, &msig, NULL);

    g_in_callback = 1;
    jstring jname = (*jni)->NewStringUTF(jni, name ? name : "<unknown>");
    jstring jsig  = (*jni)->NewStringUTF(jni, class_sig ? class_sig : "");
    (*jni)->CallStaticBooleanMethod(jni, g_xplainit_class, g_mid_on_enter, jname, jsig, (jint)0);
    if ((*jni)->ExceptionCheck(jni)) {
        (*jni)->ExceptionClear(jni);
    }
    (*jni)->DeleteLocalRef(jni, jname);
    (*jni)->DeleteLocalRef(jni, jsig);
    g_in_callback = 0;

    if (name)      (*jvmti)->Deallocate(jvmti, (unsigned char *)name);
    if (msig)      (*jvmti)->Deallocate(jvmti, (unsigned char *)msig);
    if (class_sig) (*jvmti)->Deallocate(jvmti, (unsigned char *)class_sig);
}

static void JNICALL
cb_method_exit(jvmtiEnv *jvmti, JNIEnv *jni, jthread thread, jmethodID method,
               jboolean was_popped_by_exception, jvalue return_value) {
    (void)thread; (void)was_popped_by_exception; (void)return_value;
    if (!g_bridge_ready || g_in_callback) {
        return;
    }

    char *class_sig = NULL;
    get_class_signature(jvmti, method, &class_sig);
    if (!should_trace(class_sig)) {
        if (class_sig) (*jvmti)->Deallocate(jvmti, (unsigned char *)class_sig);
        return;
    }

    char *name = NULL;
    char *msig = NULL;
    (*jvmti)->GetMethodName(jvmti, method, &name, &msig, NULL);

    g_in_callback = 1;
    jstring jname = (*jni)->NewStringUTF(jni, name ? name : "<unknown>");
    (*jni)->CallStaticBooleanMethod(jni, g_xplainit_class, g_mid_on_exit, jname);
    if ((*jni)->ExceptionCheck(jni)) {
        (*jni)->ExceptionClear(jni);
    }
    (*jni)->DeleteLocalRef(jni, jname);
    g_in_callback = 0;

    if (name)      (*jvmti)->Deallocate(jvmti, (unsigned char *)name);
    if (msig)      (*jvmti)->Deallocate(jvmti, (unsigned char *)msig);
    if (class_sig) (*jvmti)->Deallocate(jvmti, (unsigned char *)class_sig);
}

static void JNICALL
cb_exception(jvmtiEnv *jvmti, JNIEnv *jni, jthread thread, jmethodID method,
             jlocation location, jobject exception, jmethodID catch_method,
             jlocation catch_location) {
    (void)location; (void)catch_method; (void)catch_location; (void)thread;
    if (!g_bridge_ready || g_in_callback) {
        return;
    }

    /* Only report exceptions thrown from within traced code. */
    char *class_sig = NULL;
    get_class_signature(jvmti, method, &class_sig);
    if (!should_trace(class_sig)) {
        if (class_sig) (*jvmti)->Deallocate(jvmti, (unsigned char *)class_sig);
        return;
    }

    /* Resolve the exception's class signature as the error type. */
    char *exc_sig = NULL;
    if (exception != NULL) {
        jclass exc_class = (*jni)->GetObjectClass(jni, exception);
        if (exc_class != NULL) {
            (*jvmti)->GetClassSignature(jvmti, exc_class, &exc_sig, NULL);
            (*jni)->DeleteLocalRef(jni, exc_class);
        }
    }

    g_in_callback = 1;
    jstring jtype = (*jni)->NewStringUTF(jni, exc_sig ? exc_sig : "Ljava/lang/Throwable;");
    jstring jmsg  = (*jni)->NewStringUTF(jni, "");
    jstring jfile = (*jni)->NewStringUTF(jni, class_sig ? class_sig : "<jvm>");
    (*jni)->CallStaticBooleanMethod(jni, g_xplainit_class, g_mid_on_exception,
                                    jtype, jmsg, jfile, (jint)0);
    if ((*jni)->ExceptionCheck(jni)) {
        (*jni)->ExceptionClear(jni);
    }
    (*jni)->DeleteLocalRef(jni, jtype);
    (*jni)->DeleteLocalRef(jni, jmsg);
    (*jni)->DeleteLocalRef(jni, jfile);
    g_in_callback = 0;

    if (exc_sig)   (*jvmti)->Deallocate(jvmti, (unsigned char *)exc_sig);
    if (class_sig) (*jvmti)->Deallocate(jvmti, (unsigned char *)class_sig);
}

/* Resolve the Java bridge class + methods and initialise the runtime. */
static void JNICALL
cb_vm_init(jvmtiEnv *jvmti, JNIEnv *jni, jthread thread) {
    (void)jvmti; (void)thread;

    jclass local = (*jni)->FindClass(jni, "io/xplainit/Xplainit");
    if (local == NULL) {
        if ((*jni)->ExceptionCheck(jni)) (*jni)->ExceptionClear(jni);
        fprintf(stderr, "[xplainit-agent] could not find io.xplainit.Xplainit\n");
        return;
    }
    g_xplainit_class = (jclass)(*jni)->NewGlobalRef(jni, local);
    (*jni)->DeleteLocalRef(jni, local);

    g_mid_init = (*jni)->GetStaticMethodID(jni, g_xplainit_class, "agentInit", "()Z");
    g_mid_on_enter = (*jni)->GetStaticMethodID(jni, g_xplainit_class, "agentOnMethodEnter",
                                               "(Ljava/lang/String;Ljava/lang/String;I)Z");
    g_mid_on_exit = (*jni)->GetStaticMethodID(jni, g_xplainit_class, "agentOnMethodExit",
                                              "(Ljava/lang/String;)Z");
    g_mid_on_exception = (*jni)->GetStaticMethodID(jni, g_xplainit_class, "agentOnException",
                                                   "(Ljava/lang/String;Ljava/lang/String;Ljava/lang/String;I)Z");

    if (!g_mid_init || !g_mid_on_enter || !g_mid_on_exit || !g_mid_on_exception) {
        if ((*jni)->ExceptionCheck(jni)) (*jni)->ExceptionClear(jni);
        fprintf(stderr, "[xplainit-agent] could not resolve bridge methods\n");
        return;
    }

    (*jni)->CallStaticBooleanMethod(jni, g_xplainit_class, g_mid_init);
    if ((*jni)->ExceptionCheck(jni)) (*jni)->ExceptionClear(jni);

    g_bridge_ready = 1;
    fprintf(stderr, "[xplainit-agent] initialised, tracing classes matching \"%s\"\n", g_prefix);
}

/* ===== Agent entry point ===== */

JNIEXPORT jint JNICALL
Agent_OnLoad(JavaVM *vm, char *options, void *reserved) {
    (void)reserved;

    if (options != NULL && options[0] != '\0') {
        /* Allow overriding the traced package prefix. Accept either a full
         * JVMTI signature prefix ("Lio/xplainit/examples/") or a bare package
         * path ("io/xplainit/examples"). */
        if (options[0] == 'L') {
            snprintf(g_prefix, sizeof(g_prefix), "%s", options);
        } else {
            snprintf(g_prefix, sizeof(g_prefix), "L%s/", options);
        }
    }

    jvmtiEnv *jvmti = NULL;
    if ((*vm)->GetEnv(vm, (void **)&jvmti, JVMTI_VERSION_1_2) != JNI_OK || jvmti == NULL) {
        fprintf(stderr, "[xplainit-agent] failed to obtain JVMTI env\n");
        return JNI_ERR;
    }

    jvmtiCapabilities caps;
    memset(&caps, 0, sizeof(caps));
    caps.can_generate_method_entry_events = 1;
    caps.can_generate_method_exit_events  = 1;
    caps.can_generate_exception_events    = 1;
    if ((*jvmti)->AddCapabilities(jvmti, &caps) != JVMTI_ERROR_NONE) {
        fprintf(stderr, "[xplainit-agent] failed to add capabilities\n");
        return JNI_ERR;
    }

    jvmtiEventCallbacks callbacks;
    memset(&callbacks, 0, sizeof(callbacks));
    callbacks.VMInit      = &cb_vm_init;
    callbacks.MethodEntry = &cb_method_entry;
    callbacks.MethodExit  = &cb_method_exit;
    callbacks.Exception   = &cb_exception;
    if ((*jvmti)->SetEventCallbacks(jvmti, &callbacks, sizeof(callbacks)) != JVMTI_ERROR_NONE) {
        fprintf(stderr, "[xplainit-agent] failed to set callbacks\n");
        return JNI_ERR;
    }

    (*jvmti)->SetEventNotificationMode(jvmti, JVMTI_ENABLE, JVMTI_EVENT_VM_INIT, NULL);
    (*jvmti)->SetEventNotificationMode(jvmti, JVMTI_ENABLE, JVMTI_EVENT_METHOD_ENTRY, NULL);
    (*jvmti)->SetEventNotificationMode(jvmti, JVMTI_ENABLE, JVMTI_EVENT_METHOD_EXIT, NULL);
    (*jvmti)->SetEventNotificationMode(jvmti, JVMTI_ENABLE, JVMTI_EVENT_EXCEPTION, NULL);

    fprintf(stderr, "[xplainit-agent] loaded\n");
    return JNI_OK;
}

JNIEXPORT void JNICALL
Agent_OnUnload(JavaVM *vm) {
    (void)vm;
}
