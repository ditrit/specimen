import { readFileSync } from "fs"
import { join } from "path"

import type { File } from "./structure"

export type { File }

export function readLocalFile(path: string, dirname: string): File {
    let content = readFileSync(join(dirname, path), "utf-8")
    return { path, content }
}
