import * as yaml from "yaml"

import { YAMLNode } from "../structure"

export function isString(node: YAMLNode) {
    let result = yaml.isScalar(node) && typeof node.value === "string"
    return result
}
