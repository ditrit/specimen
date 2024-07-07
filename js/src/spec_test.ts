import * as yaml from "yaml"

import { iolessRun, run, readLocalFile, SpecimenContext } from "./index"

function indent(s: string, n: number): string {
    let indent = " ".repeat(n)
    return indent + s.replaceAll("\n", "\n" + indent)
}

function callLogger(s: SpecimenContext, tile: Record<string, string>) {
    const callArray = yaml.parse(tile.calls)
    let index = 0
    let error = ""

    iolessRun(
        (ss, specTile) => {
            if (error === "") {
                const expectedTile = callArray[index]
                let same = true
                if (
                    Object.keys(expectedTile).length + 1 !==
                    Object.keys(specTile).length
                ) {
                    same = false
                }
                if (same) {
                    Object.entries(specTile).forEach(([k, v]) => {
                        if (k === "filepath") {
                            return
                        }
                        if (expectedTile[k] !== v) {
                            same = false
                        }
                    })
                }
                if (!same) {
                    error = `[Call ${index}]\nExpected: ${JSON.stringify(
                        expectedTile,
                    )}\nActual__: ${JSON.stringify(specTile)}`
                }
            }
            index += 1
        },
        [{ path: tile.filepath, content: tile.spec }],
        { write: () => {} },
    )

    if (error !== "") {
        s.fail(error)
        return
    }

    if (index !== callArray.length) {
        s.fail(`Expected ${callArray.length} calls, but got ${index}`)
    }
}

function report(s: SpecimenContext, tile: Record<string, string>) {
    const behavior = yaml.parse(tile.behavior)
    let buffer: string[] = []

    iolessRun(
        (ss, specTile) => {
            const outcome = behavior[specTile.letter]
            const action =
                {
                    pass: () => {},
                    fail: () => {
                        ss.fail("failure")
                    },
                    abort: () => {
                        ss.abort("aborted")
                    },
                }[outcome] ??
                (() => {
                    ss.abort(
                        `unknown outcome: '${outcome}', specTile: ${JSON.stringify(
                            specTile,
                        )}`,
                    )
                })

            action()
        },
        [
            {
                path: tile.filepath,
                content: tile.spec,
            },
        ],
        {
            write: (s) => {
                buffer.push(s)
            },
        },
    )

    const out = buffer.join("")

    if (!out.match(new RegExp(`^${tile.report}$`))) {
        s.fail(
            `Expected:\n${indent(tile.report, 4)}\nActual:\n${indent(out, 4)}`,
        )
    }
}

run(
    (s, tile) => {
        let testbox =
            {
                "call-logger": callLogger,
                "report": report,
            }[tile.box] ??
            (() => {
                s.abort(`unknown box: ${tile.box}`)
            })

        testbox(s, tile)
    },
    [
        readLocalFile("../../spec/about.yaml", __dirname),
        readLocalFile("../../spec/flag.yaml", __dirname),
        readLocalFile("../../spec/matrix.yaml", __dirname),
        readLocalFile("../../spec/report.yaml", __dirname),
    ],
)
