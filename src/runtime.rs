use std::{cell::RefCell, io::Error, rc::Rc};

use tokio::sync::oneshot::{self, Receiver};
use v8;
use futures::futures::poll_fn;
use std::task::Context;
use crate::runner;

pub struct ModEvaluate {
    promise: v8::Global<v8::Promise>,
    sender: oneshot::Sender<Result<(), Error>>,
}

pub struct JTsRuntimeState {
    global_context: Option<v8::Global<v8::Context>>,
    pending_mod_evaluate: Option<ModEvaluate>,
}

pub struct JTsRuntime {
    isolate: Option<v8::OwnedIsolate>,
}

impl JTsRuntime {
    pub fn new() -> JTsRuntime {
        let platform = v8::new_default_platform(0, false).make_shared();
        v8::V8::initialize_platform(platform);
        v8::V8::initialize();

        let mut isolate = v8::Isolate::new(Default::default());
        let global_context = {
            let handle_scope = &mut v8::HandleScope::new(&mut isolate);
            let context = JTsRuntime::init_global(handle_scope);

            v8::Global::new(handle_scope, context)
        };

        isolate.set_slot(Rc::new(RefCell::new(JTsRuntimeState {
            global_context: Some(global_context),
            pending_mod_evaluate: None,
        })));

        JTsRuntime {
            isolate: Some(isolate),
        }
    }
    fn require(
        scope: &mut v8::HandleScope,
        args: v8::FunctionCallbackArguments,
        mut _rv: v8::ReturnValue,
    ) {
        if let Ok(v) = v8::Local::<v8::String>::try_from(args.get(0)) {
            println!("require!!:: {}", v.to_rust_string_lossy(scope));
            match v.to_rust_string_lossy(scope).as_str() {
                "fs" => {
                    let fs = runner::modules::fs::init(scope);
                    _rv.set(fs.into());
                }
                _ => {}
            }
        }
    }

    fn init_global<'s>(scope: &mut v8::HandleScope<'s, ()>) -> v8::Local<'s, v8::Context> {
        let scope = &mut v8::EscapableHandleScope::new(scope);

        let context = v8::Context::new(scope);
        let global = context.global(scope);
        let scope = &mut v8::ContextScope::new(scope, context);

        let console_object = v8::Object::new(scope);
        let console_key = v8::String::new(scope, "console").unwrap().into();
        global.set(scope, console_key, console_object.into());

        JTsRuntime::set_func(scope, console_object, "log", JTsRuntime::log);
        JTsRuntime::set_func(scope, console_object, "info", JTsRuntime::log);
        JTsRuntime::set_func(scope, console_object, "error", JTsRuntime::log);
        JTsRuntime::set_func(scope, console_object, "warn", JTsRuntime::log);

        JTsRuntime::set_func(scope, global, "require", JTsRuntime::require);

        scope.escape(context)
    }

    fn log(
        scope: &mut v8::HandleScope,
        args: v8::FunctionCallbackArguments,
        mut _rv: v8::ReturnValue,
    ) {
        let result = serde_v8::from_v8(scope, args).unwrap();
        println!("{result:?}");
        // for i in 0..args.length() {

        //     if let Ok(v) = v8::Local::<v8::String>::try_from(args.get(i)) {
        //         print!("{}", v.to_rust_string_lossy(scope));
        //     }
        //     if let Ok(v) = v8::Local::<v8::Boolean>::try_from(args.get(i)) {
        //         print!("{}", v.to_rust_string_lossy(scope));
        //     }
        //     if let Ok(v) = v8::Local::<v8::Number>::try_from(args.get(i)) {
        //         print!("{}", v.to_rust_string_lossy(scope));
        //     }
        //     print!(" ");
        // }
        print!("\n");
    }
    pub fn set_func(
        scope: &mut v8::HandleScope,
        obj: v8::Local<v8::Object>,
        name: &str,
        func: impl v8::MapFnTo<v8::FunctionCallback>,
    ) {
        let tml = v8::FunctionTemplate::new(scope, func);

        let val = tml.get_function(scope).unwrap();
        let print_name = v8::String::new(scope, name).unwrap();
        val.set_name(print_name);
        obj.set(scope, print_name.into(), val.into());
    }

    pub fn mod_evaluate(&mut self, code: String) -> Receiver<Result<(), Error>> {
        let isolate = self.isolate.as_mut().unwrap();
        let state = isolate
            .get_slot::<Rc<RefCell<JTsRuntimeState>>>()
            .unwrap()
            .clone();

        let context = state.borrow().global_context.clone().unwrap();
        let scope = &mut v8::HandleScope::with_context(isolate, context);

        let source = v8::String::new(scope, &code).unwrap();
        let name = v8::String::new(scope, "main").unwrap();
        let origin = v8::ScriptOrigin::new(
            scope,
            name.into(),
            0,
            0,
            false,
            123,
            name.into(),
            false,
            false,
            true,
        );
        let source = v8::script_compiler::Source::new(source, Some(&origin));
        let module = v8::script_compiler::compile_module(scope, source).unwrap();
        // let name = v8::String::new(scope, name).unwrap();

        let exec_scope = &mut v8::TryCatch::new(scope);
        // let script = v8::Script::compile(exec_scope, source, None).unwrap();
        // module.set_synthetic_module_export(scope, exports_key, export_value);
        let result = module.instantiate_module(exec_scope, Self::resolve_callback);
        match result {
            Some(_) => {
                println!("instantiate_module success");
            }
            None => {
                println!("instantiate_module error");
            }
        };
        let (sender, receiver) = oneshot::channel();
        // module.get_status();
        match module.evaluate(exec_scope) {
            Some(value) => {
                let promise = v8::Local::<v8::Promise>::try_from(value)
                    .expect("Expected to get promise as module evaluation result");
                state.borrow_mut().pending_mod_evaluate = Some(ModEvaluate {
                    promise: v8::Global::new(exec_scope, promise),
                    sender,
                });
                exec_scope.perform_microtask_checkpoint();
                // Some(v8::Global::new(exec_scope, value))
            }
            None => {
                assert!(exec_scope.has_caught());
                let exception = exec_scope.exception().unwrap();
                let message = exception.to_rust_string_lossy(exec_scope);
                println!("{}", message);
                // None
            }
        };
        receiver
    }
    pub fn run_event_loop(&mut self){
        poll_fn(|ctx: &mut Context|{
            let isolate = self.isolate.as_mut().unwrap();
            let state = isolate
                .get_slot::<Rc<RefCell<JTsRuntimeState>>>()
                .unwrap()
                .clone();

            {
                state.borrow().waker.register(ctx.waker());
            }

        })
    }
    fn resolve_callback<'s>(
        context: v8::Local<'s, v8::Context>,
        specifier: v8::Local<'s, v8::String>,
        import_assertions: v8::Local<'s, v8::FixedArray>,
        referrer: v8::Local<'s, v8::Module>,
    ) -> Option<v8::Local<'s, v8::Module>> {
        let scope = &mut unsafe { v8::CallbackScope::new(context) };
        println!("specifier: {}", specifier.to_rust_string_lossy(scope));
        None
    }
}

impl Default for JTsRuntime {
    fn default() -> Self {
        Self::new()
    }
}
