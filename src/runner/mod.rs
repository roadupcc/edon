pub(crate) mod modules;

// fn require(
//     scope: &mut v8::HandleScope,
//     args: v8::FunctionCallbackArguments,
//     mut _rv: v8::ReturnValue,
// ) {
//     if let Ok(v) = v8::Local::<v8::String>::try_from(args.get(0)) {
//         println!("require!!:: {}", v.to_rust_string_lossy(scope));
//         match v.to_rust_string_lossy(scope).as_str() {
//             "fs" => {
//                 let fs = modules::fs::init(scope);
//                 _rv.set(fs.into());
//             }
//             _ => {}
//         }
//     }
// }

// pub fn init_global<'s>() -> (&'s mut v8::HandleScope<'s>, v8::Local<'s, v8::Object>) {
//     let platform = v8::new_default_platform(0, false).make_shared();
//     v8::V8::initialize_platform(platform);
//     v8::V8::initialize();

//     let isolate = &mut v8::Isolate::new(Default::default());
//     let handle_scope = &mut v8::HandleScope::new(isolate);
//     let context = v8::Context::new(handle_scope);

//     // let scope = &mut v8::EscapableHandleScope::new(handle_scope);

//     let global = context.global(handle_scope);

//     let scope = &mut v8::ContextScope::new(handle_scope, context);
//     let object = v8::Object::new(scope);

//     let console_key = v8::String::new(scope, "console").unwrap().into();
//     global.set(scope, console_key, object.into());

//     set_func(scope, global, "require", require);

//     return (handle_scope, global);
// }

// pub fn set_func(
//     scope: &mut v8::HandleScope,
//     obj: v8::Local<v8::Object>,
//     name: &str,
//     func: impl v8::MapFnTo<v8::FunctionCallback>,
// ) {
//     let tml = v8::FunctionTemplate::new(scope, func);

//     let val = tml.get_function(scope).unwrap();
//     let print_name = v8::String::new(scope, name).unwrap();
//     val.set_name(print_name);
//     obj.set(scope, print_name.into(), val.into());
// }
