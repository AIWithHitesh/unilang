// Licensed to the Apache Software Foundation (ASF) under one
// or more contributor license agreements.

//! UniLang VM builtin registration for the JVM and CPython bridges.
//!
//! Call [`register_jvm_builtins`] or [`register_python_builtins`] once at VM
//! startup to make all bridge functions available as native builtins inside
//! UniLang scripts.

use std::sync::{Arc, Mutex};

use unilang_runtime::error::{ErrorKind, RuntimeError};
use unilang_runtime::value::RuntimeValue;
use unilang_runtime::vm::VM;

use crate::cpython::CpythonBridge;
use crate::jvm::JvmBridge;
use crate::thread_pool::JavaThreadPool;
use crate::types::{bridge_to_runtime, runtime_to_bridge};

// ── Helpers ───────────────────────────────────────────────────────────────────

fn bridge_err_to_runtime(e: crate::error::BridgeError) -> RuntimeError {
    RuntimeError::new(ErrorKind::Exception, e.to_string())
}

// ── JVM builtins ──────────────────────────────────────────────────────────────

/// Register all JVM bridge builtins into `vm`.
///
/// The following native functions are added to the VM's global scope:
///
/// | Function | Description |
/// |---|---|
/// | `java_import(class)` | Resolve a class and return an integer handle |
/// | `java_call_static(class, method, …args)` | Call a static method |
/// | `java_new(class, …args)` | Construct a new JVM object |
/// | `java_call(handle, method, …args)` | Call an instance method |
/// | `java_field(handle, field)` | Read an instance field |
/// | `java_load_jar(path)` | Add a JAR to the classpath |
/// | `java_thread_pool_new(threads)` | Create a fixed thread pool |
/// | `java_thread_pool_submit(pool, class, method, …args)` | Submit a task |
/// | `java_thread_pool_await(future)` | Await a submitted task |
pub fn register_jvm_builtins(vm: &mut VM) {
    let bridge: Arc<Mutex<Option<JvmBridge>>> = Arc::new(Mutex::new(JvmBridge::new().ok()));
    let pools: Arc<Mutex<std::collections::HashMap<u64, JavaThreadPool>>> =
        Arc::new(Mutex::new(std::collections::HashMap::new()));
    let pool_counter = Arc::new(std::sync::atomic::AtomicU64::new(1));

    // java_import(class_name) -> Int (handle)
    {
        let b = Arc::clone(&bridge);
        vm.register_builtin("java_import", move |args| {
            let class = extract_string(args, 0, "java_import")?;
            let handle = with_jvm(&b, |jvm| jvm.import_class(class))?;
            Ok(RuntimeValue::Int(handle as i64))
        });
    }

    // java_call_static(class, method, arg0, …) -> value
    {
        let b = Arc::clone(&bridge);
        vm.register_builtin("java_call_static", move |args| {
            if args.len() < 2 {
                return Err(RuntimeError::type_error(
                    "java_call_static requires at least (class, method)",
                ));
            }
            let class = extract_string(args, 0, "java_call_static")?;
            let method = extract_string(args, 1, "java_call_static")?;
            let bridge_args: Vec<_> = args[2..].iter().map(runtime_to_bridge).collect();
            let result = with_jvm(&b, |jvm| jvm.call_static(class, method, &bridge_args))?;
            Ok(bridge_to_runtime(result))
        });
    }

    // java_new(class, arg0, …) -> Int (handle)
    {
        let b = Arc::clone(&bridge);
        vm.register_builtin("java_new", move |args| {
            let class = extract_string(args, 0, "java_new")?;
            let bridge_args: Vec<_> = args[1..].iter().map(runtime_to_bridge).collect();
            let handle = with_jvm(&b, |jvm| jvm.new_instance(class, &bridge_args))?;
            Ok(RuntimeValue::Int(handle as i64))
        });
    }

    // java_call(handle, method, arg0, …) -> value
    {
        let b = Arc::clone(&bridge);
        vm.register_builtin("java_call", move |args| {
            if args.len() < 2 {
                return Err(RuntimeError::type_error(
                    "java_call requires at least (handle, method)",
                ));
            }
            let handle = extract_int(args, 0, "java_call")? as u64;
            let method = extract_string(args, 1, "java_call")?;
            let bridge_args: Vec<_> = args[2..].iter().map(runtime_to_bridge).collect();
            let result = with_jvm(&b, |jvm| jvm.call_instance(handle, method, &bridge_args))?;
            Ok(bridge_to_runtime(result))
        });
    }

    // java_field(handle, field) -> value
    {
        let b = Arc::clone(&bridge);
        vm.register_builtin("java_field", move |args| {
            let handle = extract_int(args, 0, "java_field")? as u64;
            let field = extract_string(args, 1, "java_field")?;
            let result = with_jvm(&b, |jvm| jvm.get_field(handle, field))?;
            Ok(bridge_to_runtime(result))
        });
    }

    // java_load_jar(path) -> Null
    {
        let b = Arc::clone(&bridge);
        vm.register_builtin("java_load_jar", move |args| {
            let path = extract_string(args, 0, "java_load_jar")?;
            with_jvm(&b, |jvm| jvm.load_jar(path))?;
            Ok(RuntimeValue::Null)
        });
    }

    // java_thread_pool_new(threads) -> Int (pool_handle)
    {
        let ps = Arc::clone(&pools);
        let pc = Arc::clone(&pool_counter);
        vm.register_builtin("java_thread_pool_new", move |args| {
            let threads = extract_int(args, 0, "java_thread_pool_new")? as usize;
            let pool = JavaThreadPool::new(threads).map_err(bridge_err_to_runtime)?;
            let id = pc.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            ps.lock()
                .map_err(|_| RuntimeError::new(ErrorKind::Exception, "pool table poisoned"))?
                .insert(id, pool);
            Ok(RuntimeValue::Int(id as i64))
        });
    }

    // java_thread_pool_submit(pool_handle, class, method, arg0, …) -> Int (future_handle)
    {
        let ps = Arc::clone(&pools);
        vm.register_builtin("java_thread_pool_submit", move |args| {
            if args.len() < 3 {
                return Err(RuntimeError::type_error(
                    "java_thread_pool_submit requires at least (pool, class, method)",
                ));
            }
            let pool_id = extract_int(args, 0, "java_thread_pool_submit")? as u64;
            let class = extract_string(args, 1, "java_thread_pool_submit")?;
            let method = extract_string(args, 2, "java_thread_pool_submit")?;
            let bridge_args: Vec<_> = args[3..].iter().map(runtime_to_bridge).collect();

            let table = ps
                .lock()
                .map_err(|_| RuntimeError::new(ErrorKind::Exception, "pool table poisoned"))?;
            let pool = table.get(&pool_id).ok_or_else(|| {
                RuntimeError::new(
                    ErrorKind::Exception,
                    format!("no thread pool with handle {}", pool_id),
                )
            })?;
            let fid = pool
                .submit(class, method, &bridge_args)
                .map_err(bridge_err_to_runtime)?;
            Ok(RuntimeValue::Int(fid as i64))
        });
    }

    // java_thread_pool_await(pool_handle, future_handle) -> value
    {
        let ps = Arc::clone(&pools);
        vm.register_builtin("java_thread_pool_await", move |args| {
            let pool_id = extract_int(args, 0, "java_thread_pool_await")? as u64;
            let future_id = extract_int(args, 1, "java_thread_pool_await")? as u64;

            let table = ps
                .lock()
                .map_err(|_| RuntimeError::new(ErrorKind::Exception, "pool table poisoned"))?;
            let pool = table.get(&pool_id).ok_or_else(|| {
                RuntimeError::new(
                    ErrorKind::Exception,
                    format!("no thread pool with handle {}", pool_id),
                )
            })?;
            let result = pool
                .await_result(future_id)
                .map_err(bridge_err_to_runtime)?;
            Ok(bridge_to_runtime(result))
        });
    }
}

// ── CPython builtins ──────────────────────────────────────────────────────────

/// Register all CPython bridge builtins into `vm`.
///
/// The following native functions are added to the VM's global scope:
///
/// | Function | Description |
/// |---|---|
/// | `py_import(module)` | Import a Python module, return Int handle |
/// | `py_call(mod_handle, func, …args)` | Call a module-level function |
/// | `py_method(obj_handle, method, …args)` | Call a method on an object |
/// | `py_getattr(obj_handle, attr)` | Read an attribute |
/// | `py_eval(code)` | Evaluate an expression string |
/// | `py_exec(code)` | Execute a statement string |
/// | `py_path_add(path)` | Append a directory to `sys.path` |
/// | `py_numpy_array(dtype, shape_list)` | Create a `SharedArrayBuffer` |
pub fn register_python_builtins(vm: &mut VM) {
    let bridge: Arc<Mutex<Option<CpythonBridge>>> = Arc::new(Mutex::new(CpythonBridge::new().ok()));

    // py_import(module) -> Int (handle)
    {
        let b = Arc::clone(&bridge);
        vm.register_builtin("py_import", move |args| {
            let module = extract_string(args, 0, "py_import")?;
            let handle = with_cpython(&b, |py| py.import_module(module))?;
            Ok(RuntimeValue::Int(handle as i64))
        });
    }

    // py_call(module_handle, func, arg0, …) -> value
    {
        let b = Arc::clone(&bridge);
        vm.register_builtin("py_call", move |args| {
            if args.len() < 2 {
                return Err(RuntimeError::type_error(
                    "py_call requires at least (module_handle, func)",
                ));
            }
            let handle = extract_int(args, 0, "py_call")? as u64;
            let func = extract_string(args, 1, "py_call")?;
            let bridge_args: Vec<_> = args[2..].iter().map(runtime_to_bridge).collect();
            let result = with_cpython(&b, |py| py.call_function(handle, func, &bridge_args))?;
            Ok(bridge_to_runtime(result))
        });
    }

    // py_method(obj_handle, method, arg0, …) -> value
    {
        let b = Arc::clone(&bridge);
        vm.register_builtin("py_method", move |args| {
            if args.len() < 2 {
                return Err(RuntimeError::type_error(
                    "py_method requires at least (obj_handle, method)",
                ));
            }
            let handle = extract_int(args, 0, "py_method")? as u64;
            let method = extract_string(args, 1, "py_method")?;
            let bridge_args: Vec<_> = args[2..].iter().map(runtime_to_bridge).collect();
            let result = with_cpython(&b, |py| py.call_method(handle, method, &bridge_args))?;
            Ok(bridge_to_runtime(result))
        });
    }

    // py_getattr(obj_handle, attr) -> value
    {
        let b = Arc::clone(&bridge);
        vm.register_builtin("py_getattr", move |args| {
            let handle = extract_int(args, 0, "py_getattr")? as u64;
            let attr = extract_string(args, 1, "py_getattr")?;
            let result = with_cpython(&b, |py| py.get_attribute(handle, attr))?;
            Ok(bridge_to_runtime(result))
        });
    }

    // py_eval(code) -> value
    {
        let b = Arc::clone(&bridge);
        vm.register_builtin("py_eval", move |args| {
            let code = extract_string(args, 0, "py_eval")?;
            let result = with_cpython(&b, |py| py.eval(code))?;
            Ok(bridge_to_runtime(result))
        });
    }

    // py_exec(code) -> Null
    {
        let b = Arc::clone(&bridge);
        vm.register_builtin("py_exec", move |args| {
            let code = extract_string(args, 0, "py_exec")?;
            with_cpython(&b, |py| py.exec(code))?;
            Ok(RuntimeValue::Null)
        });
    }

    // py_path_add(path) -> Null
    {
        let b = Arc::clone(&bridge);
        vm.register_builtin("py_path_add", move |args| {
            let path = extract_string(args, 0, "py_path_add")?;
            with_cpython(&b, |py| py.add_to_sys_path(path))?;
            Ok(RuntimeValue::Null)
        });
    }

    // py_numpy_array(dtype_str, shape_list) -> String (descriptor; real impl returns array handle)
    {
        vm.register_builtin("py_numpy_array", move |args| {
            use crate::arrays::{ArrayDtype, SharedArrayBuffer};

            let dtype_str = extract_string(args, 0, "py_numpy_array")?;
            let dtype = match dtype_str {
                "int8" => ArrayDtype::Int8,
                "int16" => ArrayDtype::Int16,
                "int32" => ArrayDtype::Int32,
                "int64" => ArrayDtype::Int64,
                "float32" => ArrayDtype::Float32,
                "float64" => ArrayDtype::Float64,
                other => {
                    return Err(RuntimeError::type_error(format!(
                        "unknown numpy dtype '{}'",
                        other
                    )))
                }
            };

            let shape = if args.len() > 1 {
                match &args[1] {
                    RuntimeValue::List(items) => items
                        .iter()
                        .map(|v| {
                            v.as_int().map(|n| n as usize).ok_or_else(|| {
                                RuntimeError::type_error("shape elements must be integers")
                            })
                        })
                        .collect::<Result<Vec<usize>, _>>()?,
                    other => {
                        return Err(RuntimeError::type_error(format!(
                            "py_numpy_array: expected list for shape, got {}",
                            other
                        )))
                    }
                }
            } else {
                vec![0]
            };

            let buf = SharedArrayBuffer::new(dtype, shape);
            Ok(RuntimeValue::String(format!(
                "<SharedArrayBuffer dtype={} len={}>",
                dtype_str,
                buf.len()
            )))
        });
    }
}

// ── Private helpers ───────────────────────────────────────────────────────────

fn extract_string<'a>(
    args: &'a [RuntimeValue],
    idx: usize,
    fn_name: &str,
) -> Result<&'a str, RuntimeError> {
    args.get(idx).and_then(|v| v.as_string()).ok_or_else(|| {
        RuntimeError::type_error(format!("{}: argument {} must be a string", fn_name, idx))
    })
}

fn extract_int(args: &[RuntimeValue], idx: usize, fn_name: &str) -> Result<i64, RuntimeError> {
    args.get(idx).and_then(|v| v.as_int()).ok_or_else(|| {
        RuntimeError::type_error(format!("{}: argument {} must be an integer", fn_name, idx))
    })
}

fn with_jvm<T>(
    bridge: &Arc<Mutex<Option<JvmBridge>>>,
    f: impl FnOnce(&JvmBridge) -> Result<T, crate::error::BridgeError>,
) -> Result<T, RuntimeError> {
    let guard = bridge
        .lock()
        .map_err(|_| RuntimeError::new(ErrorKind::Exception, "JVM bridge mutex poisoned"))?;
    match guard.as_ref() {
        Some(jvm) => f(jvm).map_err(bridge_err_to_runtime),
        None => Err(RuntimeError::new(
            ErrorKind::Exception,
            "JVM bridge not available: compile with '--features jvm'",
        )),
    }
}

fn with_cpython<T>(
    bridge: &Arc<Mutex<Option<CpythonBridge>>>,
    f: impl FnOnce(&CpythonBridge) -> Result<T, crate::error::BridgeError>,
) -> Result<T, RuntimeError> {
    let guard = bridge
        .lock()
        .map_err(|_| RuntimeError::new(ErrorKind::Exception, "CPython bridge mutex poisoned"))?;
    match guard.as_ref() {
        Some(py) => f(py).map_err(bridge_err_to_runtime),
        None => Err(RuntimeError::new(
            ErrorKind::Exception,
            "CPython bridge not available: compile with '--features cpython'",
        )),
    }
}
