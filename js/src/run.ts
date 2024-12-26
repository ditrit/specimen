import * as yaml from "yaml"

import { deepEqual } from "./deepEqual"
import * as focustree from "./focustree/focustree"
import { Nodule, parseFileIntoNodule } from "./nodule"
import { FailStatus, File, Writer } from "./structure"

export class SpecimenContext {
    tileCount: number = 0
    tilePassed: number = 0
    tileFailed: number = 0
    tileAborted: number = 0
    tileThrew: number = 0
    failureReport: string[] = []

    status: FailStatus = "Pristine"
    failInfo: string = ""

    fail(info: string) {
        this.status = "Failed"
        if (info) {
            this.failInfo = info
        }
    }

    abort(info: string) {
        this.status = "Aborted"
        if (info) {
            this.failInfo = info
        }
        throw new Error()
    }

    expectEqual<T>(value: T, wanted: T, context = "") {
        if (!deepEqual(value, wanted)) {
            if (context.length > 0) {
                context = `(${context}): `
            }
            this.fail(
                `${context}got <${typeof value}>${value} wanted <${typeof wanted}>${wanted}`,
            )
        }
    }
}

export function run(
    testFunction: (
        context: SpecimenContext,
        tile: Record<string, string>,
    ) => void,
    dataFileArray: File[],
) {
    iolessRun(testFunction, dataFileArray, process.stdout)
}

export function iolessRun(
    testFunction: (
        context: SpecimenContext,
        tile: Record<string, string>,
    ) => void,
    dataFileArray: File[],
    stdout: Writer,
) {
    let context: SpecimenContext = new SpecimenContext()

    let tree: Nodule[] = []
    dataFileArray.forEach((file) => {
        try {
            tree.push(parseFileIntoNodule(file))
        } catch (e) {
            console.error(`${file.path}: ${e}`)
        }
    })

    let validTree = tree.filter((nodule) => {
        let data_map = { filepath: [nodule.filePath] }
        let ok = true
        try {
            nodule.populate(data_map)
        } catch (e) {
            ok = false
            console.error(`${nodule.filePath}: ${e}`)
        }
        return ok
    })

    let noduleTree = new Nodule(
        null as any,
        "None",
        false,
        "",
        {},
        validTree,
        new yaml.LineCounter(),
    )
    let flagStat = { focusCount: 0, skipCount: 0 }
    let selectedLeaves = focustree.extractSelectedLeaves(
        noduleTree,
        flagStat,
        stdout,
    )

    let startTime = performance.now()

    selectedLeaves.forEach((leaf) => {
        let slab = leaf as Nodule
        let slabLocation = slab.getLocation()

        let index = -1
        for (let tile of slab.iterateDataMatrix()) {
            index += 1

            // Tile start
            context.status = "Pristine" as any
            context.failInfo = ""

            // Tile run
            try {
                testFunction(context, tile)
            } catch (e) {
                if (!["Aborted", "Failed"].includes(context.status)) {
                    context.status = "Threw"
                    context.failInfo = String(e)
                }
            }

            // Tile end
            context.tileCount += 1
            context[
                {
                    Pristine: "tilePassed",
                    Failed: "tileFailed",
                    Aborted: "tileAborted",
                    Threw: "tileThrew",
                }[context.status]
            ] += 1

            // summarize the failures
            if (context.status !== "Pristine") {
                let word = {
                    Failed: "FAIL",
                    Aborted: "ABORT",
                    Threw: "THROW",
                }[context.status]

                let message = `${word}[${slabLocation}][${index}]: ${context.failInfo}`

                context.failureReport.push(message)
            }
        }
    })

    let duration = performance.now() - startTime

    if (flagStat.focusCount > 0 || flagStat.skipCount > 0) {
        const messageArray: string[] = []
        if (flagStat.focusCount > 0) {
            messageArray.push(`${flagStat.focusCount} focused node(s)`)
        }
        if (flagStat.skipCount > 0) {
            messageArray.push(`${flagStat.skipCount} pending node(s)`)
        }
        stdout.write(`Encountered ${messageArray.join(" and ")}\n`)
    }

    // reporting what has been saved in s
    let outcome = "SUCCESS"
    if (context.failureReport.length > 0) {
        stdout.write(context.failureReport.join("\n") + "\n")
        outcome = "FAILURE"
    }
    stdout.write(
        `Ran ${context.tileCount} tiles in ${
            Math.floor(1000 * duration) / 1000
        }ms\n` +
            `${outcome} -- ${context.tilePassed} Passed | ${context.tileFailed} Failed | ${context.tileAborted} Aborted | ${context.tileThrew} Threw\n`,
    )
}
