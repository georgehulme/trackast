export function loadData() {
    return fetchFromDatabase();
}

export function transformData(data) {
    const cleaned = cleanData(data);
    return validateData(cleaned);
}

function fetchFromDatabase() {
    return "raw data from db";
}

function cleanData(data) {
    return data.trim();
}

function validateData(data) {
    return `validated: ${data}`;
}

export function anotherUnused() {
    console.log("Also never called");
}
