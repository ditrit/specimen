import { readFileSync } from "fs"

import { File } from "./structure"

export function readLocalFile(path: string): File {
    let content = readFileSync(path, "utf-8")
    return { path, content }
}
