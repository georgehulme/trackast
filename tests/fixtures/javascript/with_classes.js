class Calculator {
    constructor() {
        this.value = 0;
    }

    add(x) {
        this.validate();
    }

    validate() {
        if (this.value < 0) {
            throw new Error();
        }
    }
}

class Logger {
    constructor() {
        this.logs = [];
    }

    log(msg) {
        this.logs.push(msg);
    }
}
