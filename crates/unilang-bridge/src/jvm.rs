// Licensed to the Apache Software Foundation (ASF) under one
// or more contributor license agreements.

//! JVM bridge implementation for UniLang v2.0 interop.
//!
//! With the `jvm` feature enabled this module creates and manages a JVM via JNI,
//! allowing UniLang programs to call static/instance methods, construct objects,
//! and load JAR files at runtime.
//!
//! Without the `jvm` feature every method returns [`BridgeError::JvmNotAvailable`].

use crate::error::BridgeError;
use crate::types::BridgeValue;

/// A handle to an active JVM session.
pub struct JvmBridge {
    #[cfg(feature = "jvm")]
    vm: std::sync::Arc<jni::JavaVM>,
    #[cfg(feature = "jvm")]
    handles:
        std::sync::Arc<std::sync::Mutex<std::collections::HashMap<u64, jni::objects::GlobalRef>>>,
    #[cfg(feature = "jvm")]
    next_handle: std::sync::Arc<std::sync::atomic::AtomicU64>,
}

// Safety: JavaVM is Send + Sync per the JNI specification — threads attach
// independently, and GlobalRef is reference-counted inside the JVM.
#[cfg(feature = "jvm")]
unsafe impl Send for JvmBridge {}
#[cfg(feature = "jvm")]
unsafe impl Sync for JvmBridge {}

#[cfg(not(feature = "jvm"))]
unsafe impl Send for JvmBridge {}
#[cfg(not(feature = "jvm"))]
unsafe impl Sync for JvmBridge {}

impl JvmBridge {
    /// Attempt to initialise a JVM session.
    ///
    /// # Errors
    ///
    /// Returns [`BridgeError::JvmNotAvailable`] when the `jvm` feature is disabled or
    /// when JVM initialisation fails.
    pub fn new() -> Result<Self, BridgeError> {
        #[cfg(feature = "jvm")]
        {
            let args = jni::InitArgsBuilder::new()
                .version(jni::JNIVersion::V8)
                .build()
                .map_err(|e| BridgeError::JvmNotAvailable(e.to_string()))?;
            let vm =
                jni::JavaVM::new(args).map_err(|e| BridgeError::JvmNotAvailable(e.to_string()))?;
            Ok(Self {
                vm: std::sync::Arc::new(vm),
                handles: std::sync::Arc::new(std::sync::Mutex::new(
                    std::collections::HashMap::new(),
                )),
                next_handle: std::sync::Arc::new(std::sync::atomic::AtomicU64::new(1)),
            })
        }
        #[cfg(not(feature = "jvm"))]
        {
            Err(BridgeError::JvmNotAvailable(
                "compile with '--features jvm' to enable the JVM bridge".to_string(),
            ))
        }
    }

    /// Call a static method on a JVM class.
    pub fn call_static(
        &self,
        class: &str,
        method: &str,
        args: &[BridgeValue],
    ) -> Result<BridgeValue, BridgeError> {
        #[cfg(feature = "jvm")]
        {
            use crate::types::{bridge_to_jvalue, jvalue_to_bridge};
            use jni::objects::JClass;

            let mut env = self
                .vm
                .attach_current_thread()
                .map_err(|e| BridgeError::from_jni(e.into()))?;

            let jni_class = class.replace('.', "/");
            let cls: JClass = env
                .find_class(&jni_class)
                .map_err(|e| BridgeError::from_jni(e.into()))?;

            let mut jargs: Vec<jni::objects::JValueOwned> = Vec::with_capacity(args.len());
            for a in args {
                jargs.push(bridge_to_jvalue(&mut env, a)?);
            }
            let jargs_borrow: Vec<jni::objects::JValue> =
                jargs.iter().map(|v| v.borrow()).collect();

            // Build a method descriptor dynamically based on the argument count.
            let sig = build_sig(args.len());

            let result = env
                .call_static_method(cls, method, &sig, &jargs_borrow)
                .map_err(|e| {
                    check_and_clear_exception(&mut env);
                    BridgeError::from_jni(e.into())
                })?;

            check_and_clear_exception(&mut env);
            jvalue_to_bridge(&env, result)
        }
        #[cfg(not(feature = "jvm"))]
        {
            let _ = (class, method, args);
            Err(BridgeError::JvmNotAvailable(
                "compile with '--features jvm' to enable the JVM bridge".to_string(),
            ))
        }
    }

    /// Call an instance method on a JVM object identified by its opaque handle.
    pub fn call_instance(
        &self,
        handle: u64,
        method: &str,
        args: &[BridgeValue],
    ) -> Result<BridgeValue, BridgeError> {
        #[cfg(feature = "jvm")]
        {
            use crate::types::{bridge_to_jvalue, jvalue_to_bridge};

            let mut env = self
                .vm
                .attach_current_thread()
                .map_err(|e| BridgeError::from_jni(e.into()))?;

            let table = self
                .handles
                .lock()
                .map_err(|_| BridgeError::MarshalingError("handle table mutex poisoned".into()))?;
            let global_ref = table.get(&handle).ok_or_else(|| {
                BridgeError::MarshalingError(format!("no JVM object with handle {}", handle))
            })?;
            let obj = global_ref.as_obj();

            let mut jargs: Vec<jni::objects::JValueOwned> = Vec::with_capacity(args.len());
            for a in args {
                jargs.push(bridge_to_jvalue(&mut env, a)?);
            }
            let jargs_borrow: Vec<jni::objects::JValue> =
                jargs.iter().map(|v| v.borrow()).collect();

            let sig = build_sig(args.len());

            let result = env
                .call_method(obj, method, &sig, &jargs_borrow)
                .map_err(|e| {
                    check_and_clear_exception(&mut env);
                    BridgeError::from_jni(e.into())
                })?;

            check_and_clear_exception(&mut env);
            jvalue_to_bridge(&env, result)
        }
        #[cfg(not(feature = "jvm"))]
        {
            let _ = (handle, method, args);
            Err(BridgeError::JvmNotAvailable(
                "compile with '--features jvm' to enable the JVM bridge".to_string(),
            ))
        }
    }

    /// Look up a JVM class by name and return an opaque class handle.
    pub fn import_class(&self, class_name: &str) -> Result<u64, BridgeError> {
        #[cfg(feature = "jvm")]
        {
            let mut env = self
                .vm
                .attach_current_thread()
                .map_err(|e| BridgeError::from_jni(e.into()))?;

            let jni_name = class_name.replace('.', "/");
            let cls = env
                .find_class(&jni_name)
                .map_err(|e| BridgeError::from_jni(e.into()))?;

            let global = env
                .new_global_ref(cls)
                .map_err(|e| BridgeError::from_jni(e.into()))?;

            let handle = self
                .next_handle
                .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            self.handles
                .lock()
                .map_err(|_| BridgeError::MarshalingError("handle table mutex poisoned".into()))?
                .insert(handle, global);

            Ok(handle)
        }
        #[cfg(not(feature = "jvm"))]
        {
            let _ = class_name;
            Err(BridgeError::JvmNotAvailable(
                "compile with '--features jvm' to enable the JVM bridge".to_string(),
            ))
        }
    }

    /// Add a JAR file to the JVM classpath via `URLClassLoader` reflection.
    pub fn load_jar(&self, path: &str) -> Result<(), BridgeError> {
        #[cfg(feature = "jvm")]
        {
            use jni::objects::{JObject, JValue};

            let mut env = self
                .vm
                .attach_current_thread()
                .map_err(|e| BridgeError::from_jni(e.into()))?;

            // Build file:///absolute/path.jar URI string.
            let jar_uri = format!("file://{}", path);
            let uri_str = env
                .new_string(&jar_uri)
                .map_err(|e| BridgeError::from_jni(e.into()))?;

            // java.net.URL url = new java.net.URL(jar_uri);
            let url_cls = env
                .find_class("java/net/URL")
                .map_err(|e| BridgeError::from_jni(e.into()))?;
            let url_obj = env
                .new_object(
                    url_cls,
                    "(Ljava/lang/String;)V",
                    &[JValue::Object(&uri_str)],
                )
                .map_err(|e| BridgeError::from_jni(e.into()))?;

            // ClassLoader cl = ClassLoader.getSystemClassLoader();
            let cl_cls = env
                .find_class("java/lang/ClassLoader")
                .map_err(|e| BridgeError::from_jni(e.into()))?;
            let sys_cl = env
                .call_static_method(
                    cl_cls,
                    "getSystemClassLoader",
                    "()Ljava/lang/ClassLoader;",
                    &[],
                )
                .map_err(|e| BridgeError::from_jni(e.into()))?
                .l()
                .map_err(|e| BridgeError::from_jni(e.into()))?;

            // URLClassLoader ucl = (URLClassLoader) sys_cl;  addURL(url);
            let ucl_cls = env
                .find_class("java/net/URLClassLoader")
                .map_err(|e| BridgeError::from_jni(e.into()))?;
            let add_url = env
                .get_method_id(ucl_cls, "addURL", "(Ljava/net/URL;)V")
                .map_err(|e| BridgeError::from_jni(e.into()))?;

            unsafe {
                env.call_method_unchecked(
                    &sys_cl,
                    add_url,
                    jni::signature::ReturnType::Primitive(jni::signature::Primitive::Void),
                    &[jni::sys::jvalue {
                        l: url_obj.as_raw(),
                    }],
                )
                .map_err(|e| BridgeError::from_jni(e.into()))?;
            }

            check_and_clear_exception(&mut env);
            Ok(())
        }
        #[cfg(not(feature = "jvm"))]
        {
            let _ = path;
            Err(BridgeError::JvmNotAvailable(
                "compile with '--features jvm' to enable the JVM bridge".to_string(),
            ))
        }
    }

    /// Read a public field from a JVM object identified by its opaque handle.
    pub fn get_field(&self, handle: u64, field: &str) -> Result<BridgeValue, BridgeError> {
        #[cfg(feature = "jvm")]
        {
            use crate::types::jvalue_to_bridge;

            let mut env = self
                .vm
                .attach_current_thread()
                .map_err(|e| BridgeError::from_jni(e.into()))?;

            let table = self
                .handles
                .lock()
                .map_err(|_| BridgeError::MarshalingError("handle table mutex poisoned".into()))?;
            let global_ref = table.get(&handle).ok_or_else(|| {
                BridgeError::MarshalingError(format!("no JVM object with handle {}", handle))
            })?;
            let obj = global_ref.as_obj();

            // Determine the field type signature as java.lang.Object for simplicity.
            let field_id = env
                .get_field_id(
                    env.get_object_class(obj)
                        .map_err(|e| BridgeError::from_jni(e.into()))?,
                    field,
                    "Ljava/lang/Object;",
                )
                .map_err(|e| BridgeError::from_jni(e.into()))?;

            let val = unsafe {
                env.get_field_unchecked(obj, field_id, jni::signature::ReturnType::Object)
                    .map_err(|e| BridgeError::from_jni(e.into()))?
            };

            jvalue_to_bridge(&env, val)
        }
        #[cfg(not(feature = "jvm"))]
        {
            let _ = (handle, field);
            Err(BridgeError::JvmNotAvailable(
                "compile with '--features jvm' to enable the JVM bridge".to_string(),
            ))
        }
    }

    /// Create a new JVM object and return its opaque handle.
    pub fn new_instance(&self, class: &str, args: &[BridgeValue]) -> Result<u64, BridgeError> {
        #[cfg(feature = "jvm")]
        {
            use crate::types::bridge_to_jvalue;

            let mut env = self
                .vm
                .attach_current_thread()
                .map_err(|e| BridgeError::from_jni(e.into()))?;

            let jni_class = class.replace('.', "/");
            let cls = env
                .find_class(&jni_class)
                .map_err(|e| BridgeError::from_jni(e.into()))?;

            let mut jargs: Vec<jni::objects::JValueOwned> = Vec::with_capacity(args.len());
            for a in args {
                jargs.push(bridge_to_jvalue(&mut env, a)?);
            }
            let jargs_borrow: Vec<jni::objects::JValue> =
                jargs.iter().map(|v| v.borrow()).collect();

            let ctor_sig = build_ctor_sig(args.len());

            let obj = env.new_object(cls, &ctor_sig, &jargs_borrow).map_err(|e| {
                check_and_clear_exception(&mut env);
                BridgeError::from_jni(e.into())
            })?;

            let global = env
                .new_global_ref(obj)
                .map_err(|e| BridgeError::from_jni(e.into()))?;

            let handle = self
                .next_handle
                .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            self.handles
                .lock()
                .map_err(|_| BridgeError::MarshalingError("handle table mutex poisoned".into()))?
                .insert(handle, global);

            Ok(handle)
        }
        #[cfg(not(feature = "jvm"))]
        {
            let _ = (class, args);
            Err(BridgeError::JvmNotAvailable(
                "compile with '--features jvm' to enable the JVM bridge".to_string(),
            ))
        }
    }
}

// ── Signature helpers ─────────────────────────────────────────────────────────

/// Build `(Ljava/lang/Object;…)Ljava/lang/Object;` with `n` Object parameters.
#[allow(dead_code)]
fn build_sig(n: usize) -> String {
    let params = "Ljava/lang/Object;".repeat(n);
    format!("({})Ljava/lang/Object;", params)
}

/// Build `(Ljava/lang/Object;…)V` constructor signature with `n` parameters.
#[allow(dead_code)]
fn build_ctor_sig(n: usize) -> String {
    let params = "Ljava/lang/Object;".repeat(n);
    format!("({})V", params)
}

/// Clear any pending JVM exception so subsequent JNI calls do not fail.
#[cfg(feature = "jvm")]
fn check_and_clear_exception(env: &mut jni::JNIEnv) {
    if env.exception_check().unwrap_or(false) {
        let _ = env.exception_clear();
    }
}
