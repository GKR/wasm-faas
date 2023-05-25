import "wasi";
import { Console, Environ } from "as-wasi";

Console.log("Hello World!");
Console.log("");

let env = new Environ();
let variables = env.all();
for (let i = 0; i < variables.length; i++) {
  let v = variables[i];
  Console.log(`${v.key}: ${v.value}`);
}

let counter = 0;
while (counter < 10) {
  counter += 1;
}

let counter_start = 0;

//export function _start_2(a: i32, b: i32): i32 {
export function _start_2(): void {
  Console.log("_start_2(): Hello World!, counter_start: " + counter_start.toString());
  counter_start += 1;
}