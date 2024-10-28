import {Aoc} from "./aoc.mjs";

console.log("Commit", "${GIT_COMMIT}");

const MODULE_PATHS = [
    "./aoc-simd128.wasm",
    "./aoc.wasm",
];
if (window.crossOriginIsolated) {
    MODULE_PATHS.unshift("./aoc-threads.wasm");
}

let module;
for (const path of MODULE_PATHS) {
    try {
        module = await WebAssembly.compileStreaming(fetch(path));
        console.log("Using " + path);
        break;
    } catch (err) {
        if (err instanceof WebAssembly.CompileError) {
            console.warn("Compiling " + path + " failed: " + err);
        } else {
            throw err;
        }
    }
}
if (module === undefined) {
    throw new Error("Failed to load WebAssembly module");
}

const worker = new Worker("./worker.mjs", {type: "module"});
worker.postMessage(["init", module]);

const puzzles = Aoc.puzzleList(module);
/** @type {number} */
let YEAR;
/** @type {number} */
let DAY;

async function run(input, isExample, part) {
    if (worker.onmessage) {
        console.error("already running");
        return;
    }

    const element = document.querySelector("#aoc-output .content");
    element.textContent = "";
    element.classList.add("skeleton-block");

    const promise = new Promise((resolve, reject) => {
        worker.onmessage = (e) => {
            resolve(e);
        };
        worker.onerror = reject;
    });

    const start = performance.now();
    worker.postMessage(["run", YEAR, DAY, input, isExample, part !== "2", part !== "1"]);
    const {data: result} = await promise;
    const end = performance.now();

    worker.onmessage = undefined;
    worker.onerror = undefined;

    if (result.success) {
        const outputs = document.createElement("div");
        outputs.classList.add("columns", "is-flex-wrap-wrap");
        element.appendChild(outputs);

        for (const [name, output, enabled] of [
            ["Part 1", result.part1, part !== "2"],
            ["Part 2", result.part2, part !== "1"]
        ]) {
            if (!enabled) continue;

            const article = document.createElement("article");
            article.classList.add("message", "is-success", "column", "mb-0");
            outputs.appendChild(article);

            const header = document.createElement("div");
            header.classList.add("message-header");
            header.innerText = name;
            article.appendChild(header);

            const body = document.createElement("pre");
            body.classList.add("message-body", "has-text-success");
            body.innerText = output;
            article.appendChild(body);
        }

        const timeTaken = document.createElement("i");
        timeTaken.innerText = "Took " + Math.round(end - start) + " ms";
        element.appendChild(timeTaken);
    } else {
        const article = document.createElement("article");
        article.classList.add("message", "is-warning");
        element.appendChild(article);

        const body = document.createElement("pre");
        body.classList.add("message-body", "is-family-monospace", "has-text-warning");
        body.innerText = result.error;
        article.appendChild(body);
    }

    element.classList.remove("skeleton-block");
    element.scrollIntoView({behavior: "smooth"});
}

function updateNavbar() {
    document.querySelectorAll(".aoc-current-year").forEach((elem) => {
        elem.innerText = YEAR.toString();
    });

    document.querySelectorAll(".aoc-current-day").forEach((elem) => {
        elem.innerText = DAY.toString().padStart(2, "0");
    });

    const years = document.getElementById("aoc-navbar-years");
    years.textContent = "";
    for (const year of puzzles.keys()) {
        const elem = document.createElement("a");
        elem.classList.add("navbar-item");
        elem.innerText = year.toString();
        elem.href = `#${year}${puzzles.get(year).keys().next().value.toString().padStart(2, "0")}`
        years.appendChild(elem);
    }

    const days = document.getElementById("aoc-navbar-days");
    days.textContent = "";
    for (const day of puzzles.get(YEAR).keys()) {
        const elem = document.createElement("a");
        elem.classList.add("navbar-item");
        elem.innerText = day.toString().padStart(2, "0");
        elem.href = `#${YEAR}${day.toString().padStart(2, "0")}`
        days.appendChild(elem);
    }
}

function updateExamples() {
    const examples = document.getElementById("aoc-examples");
    const content = examples.querySelector(".content");
    content.innerText = "";

    const array = puzzles.get(YEAR).get(DAY);
    if (array.length === 0) {
        content.innerHTML = "<i>No examples<i/>";
        return;
    }

    for (const [i, example] of array.entries()) {
        const field = document.createElement("div");
        field.classList.add("field", "is-grouped", "is-align-items-center", "is-flex-wrap-wrap-reverse", "is-justify-content-flex-end");

        const textControl = document.createElement("div");
        textControl.classList.add("control", "is-expanded");
        field.appendChild(textControl);

        const textarea = document.createElement("pre");
        textarea.classList.add("pre", "py-2");
        textarea.readOnly = true;
        textarea.rows = Math.min(Math.max(example.input.split("\n").length, 1), 8);
        textarea.innerText = example.input;
        textControl.appendChild(textarea);

        const buttonControl = document.createElement("div");
        buttonControl.classList.add("control");
        field.appendChild(buttonControl);

        const buttons = document.createElement("div");
        buttons.classList.add("buttons", "has-addons");
        buttonControl.appendChild(buttons);

        for (const [name, part, enabled] of [
            ["Part 1", 1, example.part1],
            ["Part 2", 2, example.part2],
            ["Both", 0, example.part1 && example.part2]
        ]) {
            const button = document.createElement("button");
            button.classList.add("button");
            button.dataset.runPart = part.toString();
            button.dataset.runInput = i.toString();
            button.innerText = name;
            button.disabled = !enabled;
            buttons.appendChild(button);
        }

        content.appendChild(field);
    }
}

function dateChanged() {
    updateNavbar();
    updateExamples();

    document.querySelector("#aoc-input textarea").value = localStorage?.getItem(`aoc-input-${YEAR}-${DAY}`) ?? "";
    document.querySelector("#aoc-output .content").innerHTML = "<i>Output will appear here<i/>";
}

function hashChanged() {
    const location = document.location;
    if (location.hash.length === 7 && location.hash.match(/^#\d{4}(?:[01]\d|2[0-5])$/)) {
        const hashYear = parseInt(location.hash.slice(1, 5), 10);
        const hashDay = parseInt(location.hash.slice(5, 7), 10);
        if (puzzles.get(hashYear)?.get(hashDay) !== undefined) {
            YEAR = hashYear;
            DAY = hashDay;
            dateChanged();
            return;
        }
    }

    if (location.hash.length > 1) {
        location.hash = "";
    }
}

function saveInput() {
    localStorage?.setItem(`aoc-input-${YEAR}-${DAY}`, document.querySelector("#aoc-input textarea")?.value);
}

document.querySelectorAll(".navbar-burger").forEach(burger => {
    burger.addEventListener("click", () => {
        const target = document.getElementById(burger.dataset.target);
        target.classList.toggle("is-active");
        burger.classList.toggle("is-active");
    })
});

document.addEventListener("click", async (e) => {
    if (e.target.tagName !== "BUTTON") return;

    const part = e.target.dataset.runPart;
    const input = e.target.dataset.runInput;
    if (part === undefined || input === undefined) return;

    const example = puzzles.get(YEAR)?.get(DAY)?.[input];

    await run(
        (example?.input ?? document.querySelector("#aoc-input textarea").value).trimEnd(),
        example !== undefined,
        part
    );
});

const inputTextarea =  document.querySelector("#aoc-input textarea");
inputTextarea.addEventListener("dragover", (e) => {
    e.preventDefault();
    e.dataTransfer.dropEffect = "copy";
});
inputTextarea.addEventListener("dragenter", (e) => {
    e.preventDefault();
    inputTextarea.classList.add("is-skeleton");
});
inputTextarea.addEventListener("dragleave", (e) => {
    e.preventDefault();
    inputTextarea.classList.remove("is-skeleton");
});
inputTextarea.addEventListener("drop", (e) => {
    e.preventDefault();
    inputTextarea.classList.remove("is-skeleton");

    const file = e.dataTransfer.files[0];
    if (file !== undefined) {
        console.log("Loading input from file: ", file);

        const reader = new FileReader();
        reader.onload = (event) => {
            inputTextarea.value = event.target.result;
            saveInput();
        };
        reader.readAsText(file);

        return;
    }

    const text = e.dataTransfer.getData("text/plain");
    if (text !== undefined) {
        console.log("Loading input from drag text");
        inputTextarea.value = text;
        saveInput();

        return;
    }

    console.warn("No supported drag data");
});
inputTextarea.addEventListener("change", saveInput);
inputTextarea.addEventListener("input", saveInput);

window.addEventListener("hashchange", hashChanged);
hashChanged();

if (YEAR === undefined) {
    YEAR = [...puzzles.keys()].pop();
    DAY = [...puzzles.get(YEAR).keys()].pop();
    dateChanged();
}
