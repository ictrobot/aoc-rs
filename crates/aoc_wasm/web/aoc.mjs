/**
 * @typedef {Object} ModuleExports
 * @property {WebAssembly.Memory} memory
 * @property {(year: number, day: number, is_example: boolean, part1: boolean, part2: boolean) => number} run_puzzle
 * @property {WebAssembly.Global} INPUT
 * @property {WebAssembly.Global} PART1
 * @property {WebAssembly.Global} PART2
 * @property {number} PUZZLES
 * @property {WebAssembly.Global} [__tls_size]                          If multithreaded
 * @property {WebAssembly.Global} [__tls_align]                         If multithreaded
 * @property {WebAssembly.Global} [__tls_base]                          If multithreaded
 * @property {WebAssembly.Global} [__stack_pointer]                     If multithreaded
 * @property {(size: number, align: number) => number} [allocate_stack] If multithreaded
 * @property {() => void} [worker_thread]                               If multithreaded
 */

/**
 * @typedef {Object} ExampleInput
 * @property {string} input
 * @property {boolean} part1
 * @property {boolean} part2
 */

/**
 * @typedef {Map<number, Map<number, ExampleInput[]>>} Puzzles
 */

const BUFFER_SIZE = 1024 * 1024;

export class Aoc {
    /** @type {boolean} */
    #multithreaded;
    /** @type {WebAssembly.Module} */
    #module;
    /** @type {WebAssembly.Instance} */
    #instance;
    /** @type {WebAssembly.Memory} */
    #memory;
    /** @type {Worker[]|undefined} */
    #workers;
    /** @type {number|undefined} */
    #workerCount;

    /**
     * @param {WebAssembly.Module} module
     * @return {Puzzles}
     * */
    static puzzleList(module) {
        const section = WebAssembly.Module.customSections(module, "aoc_puzzles")[0];
        if (section === undefined) throw new Error("Missing aoc_puzzles custom section");

        const decoder = new TextDecoder();
        const dateString = decoder.decode(section);

        const years = new Map();
        for (let i = 0; i < dateString.length; i += 6) {
            const year = parseInt(dateString.slice(i, i + 4), 10);
            const day = parseInt(dateString.slice(i + 4, i + 6), 10);

            let days = years.get(year);
            if (days === undefined) {
                days = new Map();
                years.set(year, days);
            }

            days.set(day, this.exampleList(module, year, day));
        }

        return years;
    }

    /**
     * @param {WebAssembly.Module} module
     * @param {number} year
     * @param {number} day
     * @return {ExampleInput[]}
     * */
    static exampleList(module, year, day) {
        const section_name = `aoc_examples_${year}_${day}`;
        const section = WebAssembly.Module.customSections(module, section_name)[0];
        if (section === undefined) throw new Error(`Missing ${section_name} custom section`);

        const result = [];
        const decoder = new TextDecoder();

        let view = new Uint8Array(section);
        if (view.length === 1 && view[0] === 0) {
            return [];
        }

        while (view.length > 0) {
            const end = view.indexOf(0);
            result.push({
                input: decoder.decode(view.subarray(1, end)),
                part1: (view[0] & 0x10) !== 0,
                part2: (view[0] & 0x01) !== 0,
            });
            view = view.subarray(end + 1);
        }

        return result;
    }

    /**
     * @param {WebAssembly.Module} module
     * @param {{instance?: WebAssembly.Instance, workerCount?: number}} [options]
     */
    constructor(module, options) {
        options ??= {};
        const imports = WebAssembly.Module.imports(module);
        if (imports.length === 0) {
            this.#multithreaded = false;
            this.#module = module;
            this.#instance = options.instance;
            if (this.#instance) this.#memory = this.#exports.memory;
            if (options.workerCount !== undefined) throw new Error("workerCount can only be provided for multithreaded modules");
        } else if (imports.length === 1 && imports[0].module === "env" && imports[0].name === "memory" && imports[0].kind === "memory") {
            this.#multithreaded = true;
            this.#module = module;
            this.#workers = [];
            this.#workerCount = options.workerCount ?? navigator.hardwareConcurrency;
            if (options.instance) throw new Error("instance cannot be provided for multithreaded modules");
        } else {
            throw new Error("Unsupported module");
        }
    }

    newInstance() {
        this.stopInstance();

        if (this.#multithreaded) {
            console.debug("creating multithreaded instance");
            this.#memory = new WebAssembly.Memory({initial: 96, maximum: 2048, shared: true});
            this.#instance = new WebAssembly.Instance(this.#module, {env: {memory: this.#memory}});

            // Stack alignment must be at least 16 bytes.
            //
            // Only aligning the stack to 8 bytes (this.#exports.__tls_align.value at the time of writing) causes 2016
            // day 14 to inconsistently return wrong answers in release builds as the optimizer uses `i32.or` instead of
            // `i32.add` when adding on small array indexes.
            let align = Math.max(16, this.#exports.__tls_align.value);
            let tlsSize = Math.ceil(this.#exports.__tls_size.value / align) * align;
            let stackSize = Math.ceil(this.#exports.__stack_pointer.value / align) * align;

            // Use a single allocation for stack & tls, using the first stackSize bytes for the stack and the remaining
            // tlsSize bytes for thread local storage. This makes __tls_base and __stack_pointer the same value (as
            // the stack grows downwards and TLS is above __tls_base), similar to the main thread.
            //
            // Allocate all the stacks at once to avoid memory growing as workers start, which seems to cause problems.
            const stacks = [];
            for (let i = 0; i < this.#workerCount; i++) {
                stacks.push(this.#exports.allocate_stack(stackSize + tlsSize, align));
            }

            this.#workers = [];
            for (let i = 0; i < this.#workerCount; i++) {
                const worker = new Worker("./worker.mjs", {type: "module"});
                worker.postMessage(["thread", this.#module, this.#memory, stacks[i] + stackSize]);
                this.#workers.push(worker);
            }
        } else {
            console.debug("creating single-threaded instance");
            this.#instance = new WebAssembly.Instance(this.#module);
            this.#memory = this.#exports.memory;
        }
    }

    ensureInstance() {
        if (this.#instance === undefined) {
            this.newInstance();
        }
    }

    stopInstance() {
        if (this.#multithreaded && this.#workers !== undefined) {
            for (const worker of this.#workers) {
                try {
                    worker.terminate();
                } catch (e) {
                    console.warn(e);
                }
            }
            this.#workers = [];
        }

        this.#instance = undefined;
        this.#memory = undefined;
    }

    /**
     * @param {number} year
     * @param {number} day
     * @param {string} input
     * @param {boolean} [isExample]
     * @param {boolean} [part1]
     * @param {boolean} [part2]
     * @return {{success: true, part1: string, part2: string} | {success: false, error: string, stack?: string, panic_location?: string}}
     */
    run(year, day, input, isExample = false, part1 = true, part2 = true) {
        this.ensureInstance();

        let success;
        try {
            this.#write(input);
            success = this.#exports.run_puzzle(year, day, isExample, part1, part2);
        } catch (e) {
            console.error(e);
            let panic_payload = "";
            let panic_location = "";
            try {
                panic_payload = this.#read("PART1");
                panic_location = this.#read("PART2");
            } catch (e2) {
                console.warn(e2);
            }

            this.stopInstance();

            if (panic_payload.length > 0) {
                return {
                    success: false,
                    error: "panic: " + panic_payload,
                    panic_location,
                    stack: e.stack,
                }
            } else {
                return {
                    success: false,
                    error: e.toString(),
                    stack: e.stack,
                }
            }
        }

        if (success) {
            return {
                success: true,
                part1: this.#read("PART1"),
                part2: this.#read("PART2"),
            }
        } else {
            return {
                success: false,
                error: this.#read("PART1"),
            }
        }
    }

    /** @return {ModuleExports} */
    get #exports() {
        return /** @type {any} */ (this.#instance.exports);
    }

    /**
     * @param {"INPUT"|"PART1"|"PART2"} type
     * @return {Uint8Array}
     */
    #buffer(type) {
        const address = this.#exports[type].value;
        return new Uint8Array(this.#memory.buffer)
            .subarray(address, address + BUFFER_SIZE);
    }

    /** @param {string} input */
    #write(input) {
        const buffer = this.#buffer("INPUT");
        if (this.#multithreaded) {
            // Can't encode directly into SharedArrayBuffer
            const temp = new Uint8Array(BUFFER_SIZE);
            const result = new TextEncoder().encodeInto(input, temp);
            if (result.read < input.length || result.written === buffer.length) {
                throw new Error("Input string is too long");
            }
            buffer.set(temp.subarray(0, result.written));
            buffer[result.written] = 0;
        } else {
            const result = new TextEncoder().encodeInto(input, buffer);
            if (result.read < input.length || result.written === buffer.length) {
                throw new Error("Input string is too long");
            }
            buffer[result.written] = 0;
        }
    }

    /**
     * @param {"PART1"|"PART2"} type
     * @return string
     */
    #read(type) {
        let buffer = this.#buffer(type);

        const end = buffer.indexOf(0);
        if (end !== -1) {
            buffer = buffer.subarray(0, end);
        }

        if (this.#multithreaded) {
            // Can't decode directly from SharedArrayBuffer
            const temp = new Uint8Array(buffer.length);
            temp.set(buffer);
            buffer = temp;
        }

        return (new TextDecoder()).decode(buffer);
    }
}
