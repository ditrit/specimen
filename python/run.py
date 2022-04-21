import dataclasses
import unittest
from datetime import datetime
from typing import Callable

import yaml

import python.focustree as focustree
from .file import File
from .tree import Nodule, TreeRoot, FailStatus, SpecimenContext

@dataclasses.dataclass
class Codebox:
    name: str
    box_function: Callable[[dict], None]


def run(*data_file_list):
    def run_function(testcase_class: type):
        codebox_set = {}
        testcase_instance = testcase_class()
        for name in testcase_class.__dict__.keys():
            codebox_set[name] = Codebox(
                name=name,
                box_function=getattr(testcase_instance, name),
            )
        __flat_run(testcase_instance, codebox_set, data_file_list)
        return testcase_class
    return run_function


def __flat_run(t: unittest.TestCase, codebox_set: dict[str, Codebox], data_file_list: list[File]):
    s = SpecimenContext(t)

    print(t.__class__.__name__)

    tree = TreeRoot()
    for file in data_file_list:
        nodule = Nodule(file=file, kind="File")
        nodule.initialize_file()
        tree.append(nodule)

    valid_tree = TreeRoot()
    for nodule in tree:
        nodule.populate(codebox_set)
        try:
            pass
        except Exception as e:
            print("Exception", e)
        else:
            valid_tree.append(nodule)

    selected_leaves = focustree.extract_selected_leaves(valid_tree)
    start_time = datetime.now()

    # Run all the selected slab
    for slab in selected_leaves:
        # Pass the slab data to the codebox
        # - Manage the context (s, test start and test end)
        # - Recover from any panic that might arise during the codebox call
        # - Check the output if an expected output is provided
        # Nodule Start
        s.status = FailStatus.PRISTINE
        s.fail_info = []

        # Nodule Run
        slab.run_codebox(s)

        # Nodule End
        s.slab_count += 1
        attribute_name = {
            FailStatus.PRISTINE: "slab_passed",
            FailStatus.FAILED: "slab_failed",
            FailStatus.ABORTED: "slab_aborted",
            FailStatus.RAISED: "slab_failed",
        }[s.status]
        setattr(s, attribute_name, getattr(s, attribute_name) + 1)
        
        # summarize the failures
        if s.status != FailStatus.PRISTINE:
            slab_info = "%s(%s)" % (slab.name, slab.location)

            databox_info = ""
            if len(slab.name) > 0:
                databox_info = "[nodule %s]" % slab.name

            prefix = {
                FailStatus.FAILED: "FAIL%s" % databox_info,
                FailStatus.ABORTED: "ABORT",
                FailStatus.RAISED: "RAISE",
            }[s.status]

            if s.status == "Failed":
                databox_info = ""
                if len(slab.name) > 0:
                    databox_info = "[nodule %s]" % slab.name

            info = "; ".join(s.fail_info)

            message = "%s[codebox: %s][slab: %s]: %s" % (prefix, slab.codebox.name, slab_info, info)

            s.failure_report.append(message)

    duration = datetime.now() - start_time

    outcome = "SUCCESS"
    if len(s.failure_report) > 0:
        print("\n".join(s.failure_report))
        outcome = "FAILURE"
    print(
        ("Ran {} slabs in {}\n"
        "{} -- {} Passed | {} Failed | {} Aborted | {} Raised").format(
            s.slab_count, duration,
            outcome, s.slab_passed, s.slab_failed, s.slab_aborted, s.slab_failed,
        )
    )
