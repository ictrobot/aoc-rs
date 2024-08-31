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
    }
};
