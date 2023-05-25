use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::io;
use std::process::{Command, Stdio};
use std::result;
//use std::str;
//use std::fs::File;
//use std::io::{ BufRead, BufReader };

use duct::cmd;

use wasmtime::{Config, Engine, Linker, Module, Store};
use wasmtime_wasi::sync::WasiCtxBuilder;
use wasi_common::pipe::{WritePipe};

use actix_web::{get, App, HttpRequest, HttpResponse, HttpServer, Responder, Result};
use actix_web::web::{Query, Path};
use actix_files::NamedFile;
use std::path::PathBuf;

fn invoke_wasm_module(module_name: String, params: HashMap<String, String>) -> result::Result<String, wasmtime::Error> {
    let mut cfg = Config::new();
    cfg.consume_fuel(true);
    //let engine = Engine::default();
    let engine = Engine::new(&cfg)?;
    let mut linker = Linker::new(&engine);
    wasmtime_wasi::add_to_linker(&mut linker, |s| s)?;

    let stdout_buf: Vec<u8> = vec![];
    let stdout_mutex = Arc::new(RwLock::new(stdout_buf));
    let stdout = WritePipe::from_shared(stdout_mutex.clone());

    // convert params hashmap to an array
    let envs: Vec<(String,String)>  = params.iter().map(|(key, value)| {
        (key.clone(), value.clone())
    }).collect();

    let wasi = WasiCtxBuilder::new()
        .stdout(Box::new(stdout))
        .envs(&envs)?
        .build();
    let mut store = Store::new(&engine, wasi);

    store.add_fuel(100000)?; // amount of fuel limits how many instructions can be executed
    
    let module = Module::from_file(&engine, &module_name)?;
    linker.module(&mut store, &module_name, &module)?;

    let instance = linker.instantiate(&mut store, &module)?;
    let instance_main = instance.get_typed_func::<(), ()>(&mut store, "_start")?;
    instance_main.call(&mut store, ())?;

    match instance.get_typed_func::<(), ()>(&mut store, "_start_3") {
        Some(fun) => {

        },
        None => {

        }
    }
    let fun_export = instance.get_typed_func::<(), ()>(&mut store, "_start_3")?;
    fun_export.call(&mut store, ())?;
    fun_export.call(&mut store, ())?;

    // https://docs.wasmtime.dev/api/wasmtime/struct.Memory.html#method.size
    // WebAssembly memories represent a contiguous array of bytes that have a size that is 
    // always a multiple of the WebAssembly page size, currently 64 kilobytes.
    let memory = instance.get_memory(&mut store, "memory");
    match memory {
        None => println!("memory is None"),
        Some(memory_value) => println!("memory size: {}", memory_value.size(&store)),
    }

    let fuel_consumed = store.fuel_consumed();
    println!("fuel_consumed: {}", fuel_consumed.unwrap());

    let mut buffer: Vec<u8> = Vec::new();
    stdout_mutex.read().unwrap().iter().for_each(|i| {
        buffer.push(*i)
    });

    let s = String::from_utf8(buffer)?;
    Ok(s)
}

#[get("/ls")]
async fn handler_ls(_query: Query<HashMap<String, String>>) -> impl Responder {
    
    println!("/ls");

    let output = Command::new("ls")
        // Tell the OS to record the command's output
        .stdout(Stdio::piped())
        // execute the command, wait for it to complete, then capture the output
        .output()
        // Blow up if the OS was unable to start the program
        .unwrap();

    // extract the raw bytes that we captured and interpret them as a string
    let stdout = String::from_utf8(output.stdout).unwrap();

    println!("{}", stdout);
    
    HttpResponse::Ok().body(stdout)
}

#[get("/compile")]
async fn handler_compile(_query: Query<HashMap<String, String>>) -> impl Responder {
    
    // https://crates.io/crates/duct
    // https://stackoverflow.com/a/69143080
    // https://github.com/rust-lang/cargo/blob/master/src/bin/cargo/commands/build.rs

    println!("/compile");

    //let stdout = cmd!("echo", "hi").read().unwrap();
    //let stdout = cmd!("cargo", "build").read().unwrap();
    
    let output = cmd!("sh", "compile-rs.sh").stdout_to_stderr().stderr_capture().unchecked().run().unwrap();
    //let output = cmd!("cd", "../examples/hello-world-rs/", "cargo", "build").stdout_to_stderr().stderr_capture().unchecked().run().unwrap();
    //let output = cmd!("bash", "-c", "echo out && echo err 1>&2").stdout_to_stderr().stderr_capture().unchecked().run().unwrap();
    //let output = cmd!("cargo", "build").stdout_to_stderr().stderr_capture().unchecked().run().unwrap();

    /*let stderr = output.stderr;
    let s = match str::from_utf8(&output.stderr) {
        Ok(v) => v,
        Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
    };*/

    //let stdout = str::from_utf8(&output.stderr);
    /*let reader = output.stderr_to_stdout().reader().unwrap();
    let mut lines = BufReader::new(reader).lines();
    for line in lines {
        println!("{}", line.unwrap());
    }*/

    //let output_data = output.stderr.clone();
    //let val = str::from_utf8(&output_data).unwrap();

    let s = String::from_utf8(output.stderr).unwrap();
    HttpResponse::Ok().body(s)
}

#[get("/{module_name}")]
async fn handler(module_name: Path<String>, query: Query<HashMap<String, String>>)
    -> impl Responder {
    let wasm_module = format!("{}{}", module_name, ".wasm");
    if !std::path::Path::new(&wasm_module).exists() {
        return HttpResponse::NotFound().body("Module not found");
    }
    let val = invoke_wasm_module(wasm_module, query.into_inner()).expect("invocation error");
    HttpResponse::Ok().body(val)
}

async fn index(req: HttpRequest) -> Result<NamedFile> {
    let path: PathBuf = req.match_info().query("filename").parse().unwrap();
    println!("path: {}", path.display());
    if path.as_os_str().is_empty() {
        let index = PathBuf::from(r"./index.html");
        return Ok(NamedFile::open(index)?)
    }
    Ok(NamedFile::open(path)?)
}

#[actix_web::main]
async fn main() -> io::Result<()> {
    HttpServer::new(|| {
            App::new()
                .service(handler_ls)
                .service(handler_compile)
                .service(handler)
                .route("/{filename:.*}", actix_web::web::get().to(index))
        })
        .bind("127.0.0.1:8080")?
        .run()
        .await
}