function deepEqual<T>(x: T, y: T): boolean

function deepEqual<T extends {}>(x: T, y: T) {
    if (x === y) {
        return true
    } else if (typeof x == "object" && typeof y == "object" && x != null && y != null) {
        if (Object.keys(x).length != Object.keys(y).length) {
            return false
        }

        for (var prop in x) {
            if (y.hasOwnProperty(prop)) {
                if (!deepEqual(x[prop], y[prop])) {
                    return false
                }
            } else {
                return false
            }
        }

        return true
    } else {
        return false
    }
}

export { deepEqual }
