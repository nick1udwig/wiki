Use kit here: https://github.com/hyperware-ai/kit/pull/312
https://github.com/hyperware-ai/kit/pull/312/commits/a4a7ea478ba51c3e16aa13f250fce05e601189e1

```
kit b --hyperapp
```

gives

```
ERROR src/main.rs:1330:
   0: Command `cargo ["+stable", "build", "--release", "--no-default-features", "--target", "wasm32-wasip1", "--target-dir", "target", "--color=always"]` failed with exit code Some(101)
      stdout:
      stderr:    Compiling sign v0.1.0 (/home/nick/git/sign/sign)
      error: failed to resolve directory while parsing WIT for path [/home/nick/git/sign/sign/target/wit]

             Caused by:
               failed to parse package: /home/nick/git/sign/sign/target/wit

             Caused by:
               type `unknown` does not exist
                  --> /home/nick/git/sign/sign/target/wit/sign.wit:15:18
                   |
                15 |         message: unknown,
                   |                  ^------
        --> sign/src/lib.rs:75:1
         |
      75 | / #[hyperprocess(
      76 | |     name = "sign",
      77 | |     ui = None,
      78 | |     endpoints = vec![],
      79 | |     save_config = SaveOptions::Never,
      80 | |     wit_world = "sign-sys-v0",
      81 | | )]
         | |__^
         |
         = note: this error originates in the macro `wit_bindgen::generate` (in Nightly builds, run with -Z macro-backtrace for more info)

      error[E0255]: the name `Request` is defined multiple times
        --> sign/src/lib.rs:75:1
         |
      7  |       our, Address, LazyLoadBlob, Request,
         |                                   ------- previous import of the type `Request` here
      ...
      75 | / #[hyperprocess(
      76 | |     name = "sign",
      77 | |     ui = None,
      78 | |     endpoints = vec![],
      79 | |     save_config = SaveOptions::Never,
      80 | |     wit_world = "sign-sys-v0",
      81 | | )]
         | |__^ `Request` redefined here
         |
         = note: `Request` must be defined only once in the type namespace of this module
         = note: this error originates in the attribute macro `hyperprocess` (in Nightly builds, run with -Z macro-backtrace for more info)
      help: you can use `as` to change the binding name of the import
         |
      7  |     our, Address, LazyLoadBlob, Request as OtherRequest,
         |                                         +++++++++++++++

      error: cannot find macro `export` in this scope
        --> sign/src/lib.rs:75:1
         |
      75 | / #[hyperprocess(
      76 | |     name = "sign",
      77 | |     ui = None,
      78 | |     endpoints = vec![],
      79 | |     save_config = SaveOptions::Never,
      80 | |     wit_world = "sign-sys-v0",
      81 | | )]
         | |__^
         |
         = note: this error originates in the attribute macro `hyperprocess` (in Nightly builds, run with -Z macro-backtrace for more info)

      error[E0425]: cannot find value `source` in this scope
        --> sign/src/lib.rs:17:32
         |
      17 |     let message = make_message(source, &message);
         |                                ^^^^^^ not found in this scope

      error[E0425]: cannot find value `body` in this scope
        --> sign/src/lib.rs:24:15
         |
      24 |         .body(body)
         |               ^^^^ not found in this scope

      error[E0425]: cannot find value `source` in this scope
        --> sign/src/lib.rs:38:32
         |
      38 |     let message = make_message(source, &message);
         |                                ^^^^^^ not found in this scope

      error[E0405]: cannot find trait `Guest` in this scope
        --> sign/src/lib.rs:75:1
         |
      75 | / #[hyperprocess(
      76 | |     name = "sign",
      77 | |     ui = None,
      78 | |     endpoints = vec![],
      79 | |     save_config = SaveOptions::Never,
      80 | |     wit_world = "sign-sys-v0",
      81 | | )]
         | |__^ not found in this scope
         |
         = note: this error originates in the attribute macro `hyperprocess` (in Nightly builds, run with -Z macro-backtrace for more info)

      error[E0117]: only traits defined in the current crate can be implemented for types defined outside of the crate
        --> sign/src/lib.rs:75:1
         |
      75 | / #[hyperprocess(
      76 | |     name = "sign",
      77 | |     ui = None,
      78 | |     endpoints = vec![],
      79 | |     save_config = SaveOptions::Never,
      80 | |     wit_world = "sign-sys-v0",
      81 | | )]
         | |__^ `hyperware_process_lib::Request` is not defined in the current crate
         |
         = note: impl doesn't have any local type before any uncovered type parameters
         = note: for more information see https://doc.rust-lang.org/reference/items/implementations.html#orphan-rules
         = note: define and implement a trait or new type instead
         = note: this error originates in the derive macro `Debug` (in Nightly builds, run with -Z macro-backtrace for more info)

      error[E0117]: only traits defined in the current crate can be implemented for types defined outside of the crate
        --> sign/src/lib.rs:75:1
         |
      75 | / #[hyperprocess(
      76 | |     name = "sign",
      77 | |     ui = None,
      78 | |     endpoints = vec![],
      79 | |     save_config = SaveOptions::Never,
      80 | |     wit_world = "sign-sys-v0",
      81 | | )]
         | |  ^ `hyperware_process_lib::Request` is not defined in the current crate
         | |__|
         |
         |
         = note: impl doesn't have any local type before any uncovered type parameters
         = note: for more information see https://doc.rust-lang.org/reference/items/implementations.html#orphan-rules
         = note: define and implement a trait or new type instead
         = note: this error originates in the derive macro `serde::Serialize` (in Nightly builds, run with -Z macro-backtrace for more info)

      error[E0117]: only traits defined in the current crate can be implemented for types defined outside of the crate
        --> sign/src/lib.rs:75:1
         |
      75 | / #[hyperprocess(
      76 | |     name = "sign",
      77 | |     ui = None,
      78 | |     endpoints = vec![],
      79 | |     save_config = SaveOptions::Never,
      80 | |     wit_world = "sign-sys-v0",
      81 | | )]
         | |  ^ `hyperware_process_lib::Request` is not defined in the current crate
         | |__|
         |
         |
         = note: impl doesn't have any local type before any uncovered type parameters
         = note: for more information see https://doc.rust-lang.org/reference/items/implementations.html#orphan-rules
         = note: define and implement a trait or new type instead
         = note: this error originates in the derive macro `serde::Deserialize` (in Nightly builds, run with -Z macro-backtrace for more info)

      Some errors have detailed explanations: E0117, E0255, E0405, E0425.
      For more information about an error, try `rustc --explain E0117`.
      error: could not compile `sign` (lib) due to 10 previous errors
   0:

Location:
   src/build/mod.rs:215

  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ SPANTRACE ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

   0: kit::build::run_command
      at src/build/mod.rs:178
   1: kit::build::compile_rust_wasm_process
      at src/build/mod.rs:897
   2: kit::build::compile_package_item
      at src/build/mod.rs:1091

```

This indicates we need:

1. A way to get `source`,
2. A way to deal with `blob`s in `send`,
3. Support for `Vec<u8>`,
4. A non-colliding name for the macro `Request` type.
