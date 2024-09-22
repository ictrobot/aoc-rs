/**
 * @typedef {Object} ModuleExports
 * @property {WebAssembly.Memory} memory
 * @property {(year: number, day: number, is_example: boolean, part1: boolean, part2: boolean) => number} run_puzzle
 * @property {WebAssembly.Global} INPUT
 * @property {WebAssembly.Global} PART1
 * @property {WebAssembly.Global} PART2
 * @property {number} PUZZLES
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
    /** @type {WebAssembly.Module} */
    #module;
    /** @type {WebAssembly.Instance} */
    #instance;
    /** @type {Puzzles} */
    #puzzles;

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
     * @param {WebAssembly.Instance} [instance]
     */
    constructor(module, instance) {
        this.#module = module;
        this.#instance = instance ?? new WebAssembly.Instance(module);
    }

    /** @return {Puzzles} */
    get puzzles() {
        this.#puzzles ??= Aoc.puzzleList(this.#module);
        return this.#puzzles;
    }

    /**
     * @param {number} year
     * @param {number} day
     * @param {string} input
     * @param {boolean} [isExample]
     * @param {boolean} [part1]
     * @param {boolean} [part2]
     * @return {{success: true, part1: string, part2: string} | {success: false, error: string}}
     */
    run(year, day, input, isExample = false, part1 = true, part2 = true) {
        let success;
        try {
            this.#write(input);
            success = this.#exports.run_puzzle(year, day, isExample, part1, part2);
        } catch (e) {
            this.#instance = new WebAssembly.Instance(this.#module);
            return {
                success: false,
                error: "Unexpected error: " + e.toString() + (e.stack ? "\n\n" + e.stack : ""),
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
        return new Uint8Array(this.#exports.memory.buffer)
            .subarray(address, address + BUFFER_SIZE);
    }

    /** @param {string} input */
    #write(input) {
        const buffer = this.#buffer("INPUT");
        const result = new TextEncoder().encodeInto(input, buffer);
        if (result.read < input.length || result.written === buffer.length) {
            throw new Error("Input string is too long");
        }
        buffer[result.written] = 0;
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

        return (new TextDecoder()).decode(buffer);
    }
}