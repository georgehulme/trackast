import { loadData, transformData } from './utils.js';

export function mainEntry() {
    const data = loadData();
    const result = processData(data);
    outputResult(result);
}

export function processData(data) {
    return transformData(data);
}

export function outputResult(result) {
    console.log(`Result: ${result}`);
}

export function unusedFunction() {
    console.log("This function is never called");
}
