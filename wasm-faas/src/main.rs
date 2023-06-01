use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::io;
use std::process::{Command, Stdio};
use std::result;
use std::str;
use std::env;
use std::io::Write; // bring trait into scope
use std::fs;
//use std::fs::File;
//use std::io::{ BufRead, BufReader };

use duct::cmd;

use wasmtime::{Config, Engine, Linker, Module, Store};
use wasmtime_wasi::sync::WasiCtxBuilder;
use wasi_common::pipe::{WritePipe};

use actix_web::{get, web, App, HttpRequest, HttpResponse, HttpServer, Responder, Result};
use actix_web::web::{Query, Path};
use actix_files::NamedFile;
use std::path::PathBuf;

use json::JsonValue;
use serde::{Deserialize, Serialize};
use serde_json;

use futures_util::stream::StreamExt;

use base64::{Engine as _, engine::general_purpose};

use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
struct MyObj {
    name: String,
    number: i32,
    data_b64: String,
}

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
        Ok(func) => {
            func.call(&mut store, ())?;
        },
        _error => {
            println!("found no exported function named _start_3()");
        }
    }
    //let fun_export = instance.get_typed_func::<(), ()>(&mut store, "_start_3")?;
    //fun_export.call(&mut store, ())?;
    //fun_export.call(&mut store, ())?;

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
    println!("output: {}", s);

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

#[get("/module/{module_name}")]
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

const MAX_SIZE: usize = 262_144; // max payload size is 256k

/// This handler manually load request payload and parse json object
async fn index_manual(mut payload: web::Payload) -> Result<HttpResponse, actix_web::Error> {
    // payload is a stream of Bytes objects
    let mut body = web::BytesMut::new();
    
    while let Some(chunk) = payload.next().await {
        let chunk = chunk?;
        // limit max size of in-memory payload
        if (body.len() + chunk.len()) > MAX_SIZE {
            return Err(actix_web::error::ErrorBadRequest("overflow"));
        }
        body.extend_from_slice(&chunk);
    }

    // body is loaded, now we can deserialize serde-json
    let obj = serde_json::from_slice::<MyObj>(&body)?;

    let data_b64 = obj.data_b64.clone();
    let decoded_data_b64 = general_purpose::STANDARD.decode(data_b64).unwrap();
    let decoded_b64 = str::from_utf8(&decoded_data_b64);

    let mut path_to_file = String::new();
    let mut path_to_dir = String::new();
    let id = Uuid::new_v4().to_string();
    let code_dir = format!("{id}");

    let file_name = Uuid::new_v4().to_string();
    let file_name_str = format!("{file_name}");
    
    match env::current_exe() {
        Ok(exe_path) => {
            let mut exe_dir = exe_path.clone();
            exe_dir.pop();

            let code_folder = exe_dir.display().to_string();
            path_to_dir = format!("{code_folder}/{code_dir}");
            path_to_file = format!("{path_to_dir}/{file_name_str}.rs");

            println!("code_folder: {}", path_to_dir.clone());
            println!("Path of this executable is: {}", exe_path.display());
            println!("path_to_file: {}", path_to_file);
        },
        Err(e) => println!("failed to get current exe path: {e}"),
    };

    fs::create_dir_all(path_to_dir.clone())?;

    //let path_to_dir_target = format!("{path_to_dir}/target/wasm32-wasi/release/deps");
    //fs::create_dir_all(path_to_dir_target.clone())?;

    let path_to_file_compile = path_to_file.clone();
    if !path_to_file.is_empty() {
        let mut file = fs::OpenOptions::new()
            .create(true) // To create a new file
            .write(true)
            .truncate(true)
            // either use the ? operator or unwrap since it returns a Result
            .open(path_to_file)?;
        
        match file.write_all(&decoded_data_b64) {
            Ok(_result) => {
                println!("File written");
            }
            Err(e) => {
                println!("Error writing file: {}", e);
            }
        }
    }

    println!("decoded_b64: [{}]", decoded_b64.unwrap());

    //let output_dir = format!("{code_dir}");
    //let output_dir_path = PathBuf::from(&output_dir);
    //let output_dir_absolute = fs::canonicalize(&output_dir_path).unwrap();

    let output = cmd!("sh", "run-rustc.sh", path_to_file_compile, path_to_dir, "hello-test").stdout_to_stderr().stderr_capture().unchecked().run().unwrap();
    let s = String::from_utf8(output.stderr).unwrap();
    println!("run-rustc:sh: [{}]", s);

    //Ok(HttpResponse::Ok().json(obj)) // <- send response

    //let response = json::object! {"result" => s };
    Ok(HttpResponse::Ok()
        .content_type("application/json")
        .body(s))
}

/// This handler manually load request payload and parse json-rust
async fn index_mjsonrust(body: web::Bytes) -> Result<HttpResponse, actix_web::Error> {
    // body is loaded, now we can deserialize json-rust
    let result = json::parse(std::str::from_utf8(&body).unwrap()); // return Result
    let injson: JsonValue = match result {
        Ok(v) => v,
        Err(e) => json::object! {"err" => e.to_string() },
    };
    Ok(HttpResponse::Ok()
        .content_type("application/json")
        .body(injson.dump()))
}

#[actix_web::main]
async fn main() -> io::Result<()> {
    HttpServer::new(|| {
            App::new()
                .app_data(web::JsonConfig::default().limit(4096)) // <- limit size of the payload (global configuration)
                .service(handler_ls)
                .service(handler_compile)
                .service(web::resource("/api/manual").route(web::post().to(index_manual)))
                .service(web::resource("/api/mjsonrust").route(web::post().to(index_mjsonrust)))
                .service(handler)
                .route("/{filename:.*}", actix_web::web::get().to(index))
        })
        .bind("127.0.0.1:8080")?
        .run()
        .await
}