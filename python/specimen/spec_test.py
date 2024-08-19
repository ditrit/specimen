from io import StringIO
import re
import yaml

from . import run, ioless_run, read_local_file, File

def indent(text, space_count):
    return "\n".join(" " * space_count + line for line in text.split("\n"))

def call_logger(context, calls, filepath, spec, **kwargs):
    call_list = yaml.load(calls, Loader=yaml.Loader)
    index = [0]
    error = [None]

    @ioless_run(
        File(path=filepath, content=spec),
        stdout=StringIO(),
    )
    def local_test(context, filepath, **kwargs):
        if error[0] == None:
            expected_tile = call_list[index[0]]
            if expected_tile != kwargs:
                error[0] = f"[Call {index[0]}]\nExpected: {expected_tile}\nActual__: {kwargs}"
        index[0] += 1

    if error[0] != None:
        context.fail(error[0])
        return
    
    if index[0] != len(call_list):
        context.fail(f"Expected {len(call_list)} calls, but got {index[0]}")
        return

def report(context, behavior, filepath, spec, report, **kwargs):
    behaviorMap = yaml.load(behavior, Loader=yaml.Loader)
    buffer = StringIO()

    @ioless_run(
        File(path=filepath, content=spec),
        stdout=buffer,
    )
    def local_test(context, letter, **kwargs):
        outcome = behaviorMap[letter]
        if outcome == "pass":
            pass
        elif outcome == "fail":
            context.fail("failure")
        elif outcome == "abort":
            context.abort("aborted")
        else:
            context.abort(f"unknown outcome: '{outcome}', letter: {letter}, kwargs: {kwargs}")
    
    out = buffer.getvalue()

    if not re.fullmatch(report, out):
        context.fail(f"Expected:\n{indent(report, 4)}\nActual:\n{indent(out, 4)}")


@run(
    read_local_file("../../spec/about.yaml", location=__file__),
    read_local_file("../../spec/flag.yaml", location=__file__),
    read_local_file("../../spec/matrix.yaml", location=__file__),
    read_local_file("../../spec/report.yaml", location=__file__),
)
def test(context, box=None, **kwargs):
    if box == "call-logger":
        call_logger(context, **kwargs)
    elif box == "report":
        report(context, **kwargs)
    else:
        context.abort("unknown box: %s" % box)
