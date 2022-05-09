import * as yaml from "yaml"

import * as syaml from "./syaml"
import * as focustree from "./focustree/focustree"
import { BoxFunction, Codebox, FailStatus, File } from "./structure"
import { Nodule, SpecimenContext, TreeRoot } from "./tree"

export function makeCodeboxSet(
    codeboxMap: Record<string, BoxFunction>,
): Record<string, Codebox> {
    let result = {} as Record<string, Codebox>
    Object.entries(codeboxMap).forEach(([key, value]) => {
        result[key] = {
            name: key,
            boxFunction: value,
        }
    })
    return result
}

export function run(
    codeboxSet: Record<string, Codebox>,
    dataFileArray: File[],
) {
    let context: SpecimenContext = new SpecimenContext()

    var tree = new TreeRoot()
    dataFileArray.forEach((file) => {
        // Parsing the data into a tree of nodules
        // /\ initializeFile
        let lineCounter = new yaml.LineCounter()
        let document = yaml.parseDocument(file.content, { lineCounter })
        let mapping: yaml.Node = document.contents!
        syaml.assertIsMap<yaml.YAMLMap<string, any>>(mapping)
        let nodule = new Nodule(file, "File", lineCounter, document, mapping)
        nodule.initialize()
        // \/

        // Populating input and codebox fields
        try {
            nodule.populate(codeboxSet, undefined, {}, {})
            tree.push(nodule)
        } catch (e) {
            console.error(e)
        }
    })

    let selectedLeaves = focustree.extractSelectedLeaves(tree)

    let startTime = performance.now()

    selectedLeaves.forEach((leaf) => {
        let slab = leaf as Nodule

        context.status = "Pristine" as any
        context.failInfo = []

        slab.runCodebox(context)

        context.slabCount += 1
        switch (context.status) {
            case "Pristine":
                context.slabPassed += 1
                break
            case "Failed":
                context.slabFailed += 1
                break
            case "Aborted":
                context.slabAborted += 1
                break
            case "Threw":
                context.slabThrew += 1
                break
        }

        // summarize the failures
        if (context.status !== "Pristine") {
            let slabInfo = `${slab.name}(${slab.location})`
            let info = context.failInfo.join("; ")
            let message = ""
            switch (context.status) {
                case "Failed":
                    let nameInfo = slab.name ? `[nodule ${slab.name}]` : ""
                    message = `FAIL${nameInfo}`
                    break
                case "Aborted":
                    message = "ABORT"
                case "Threw":
                    message = "THROW"
            }

            message = `${message}[codebox: ${slab.codebox?.name}][slab: ${slabInfo}]: ${info}`

            context.failureReport.push(message)
        }
    })

    let duration = performance.now() - startTime

    // reporting what has been saved in s
    let outcome = "SUCCESS"
    if (context.failureReport.length > 0) {
        console.log(context.failureReport.join("\n"))
        outcome = "FAILURE"
    }
    console.log(
        `Ran ${context.slabCount} slabs in ${duration} ms\n` +
            `${outcome} -- ${context.slabPassed} Passed | ${context.slabFailed} Failed | ${context.slabAborted} Aborted | ${context.slabThrew} Threw`,
    )
}
