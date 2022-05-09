import { readFileSync } from "fs"
import { join } from "path"

import { File } from "./structure"

export function readLocalFile(path: string, dirname: string): File {
    let content = readFileSync(join(dirname, path), "utf-8")
    return { path, content }
}
