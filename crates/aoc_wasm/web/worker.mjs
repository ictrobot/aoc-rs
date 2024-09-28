import {Aoc} from "./aoc.mjs";

let instance;

onmessage = (e) => {
    console.log("worker: ", e.data);
    switch (e.data.shift()) {
        case "init":
            instance = new Aoc(...e.data);
            break;
        case "run":
            console.time("solution");
            const result = instance.run(...e.data);
            console.timeEnd("solution");
            postMessage(result);
            console.log(result);
            break;
        case "thread":
            const [module, memory, ptr] = e.data;
            instance = new WebAssembly.Instance(module, {env: {memory}});
            instance.exports.__stack_pointer.value = ptr; // Stack uses storage below the provided pointer
            instance.exports.__wasm_init_tls(ptr); // TLS uses storage above the provided pointer
            instance.exports.worker_thread();
            throw new Error("unreachable");
    }
};
