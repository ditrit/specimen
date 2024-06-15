import * as yaml from "yaml"

import * as focustree from "./focustree/focustree"

export function readFlag(node: yaml.YAMLMap<any, any>): focustree.FlagType {
    let flagEntry = node.get("flag")
    let flag: focustree.FlagType = "None"

    if (flagEntry === undefined) {
        return flag
    }

    let name = ""
    let both = false
    let sentence = (flagEntry as string).split(" ")
    sentence.forEach((word) => {
        if (word === "FOCUS") {
            if (flag === "Skip") {
                both = true
            }
            flag = "Focus"
            name = word
        } else if (word === "PENDING") {
            if (flag === "Focus") {
                both = true
            }
            flag = "Skip"
            name = word
        } else if (word.toUpperCase() === word) {
            console.warn(
                `Unrecognized all uppercase flag \"${word}\". It has been ignored.`,
            )
        }
    })
    if (both) {
        console.warn(
            `Both FOCUS and PENDING have been found among the flags of a node. ${name} has been kept.`,
        )
    }
    return flag
}