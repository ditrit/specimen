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
        nodule = Nodule(file=file, kind="file")
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
        s.status = "Pristine"
        s.fail_info = []

        # Nodule Run
        slab.run_codebox(s)

        # Nodule End
        s.slab_count += 1
        attribute_name = dict(
            Pristine="slab_passed",
            Failed="slab_failed",
            Aborted="slab_aborted",
            Panicked="slab_panicked",
        )[s.status]
        setattr(s, attribute_name, getattr(s, attribute_name) + 1)
        
        # summarize the failures
        if s.status != "Pristine":
            slab_info = "%s(%s)" % (slab.Name, slab.Location)

            info = "; ".join(s.failInfo)

            prefix = dict(
            Failed="FAIL%s" % databox_info,
            Aborted="ABORT",
            Panicked="PANIC",
            )[s.status]

            if s.status == "Failed":
                databox_info = ""
                if len(slab.name) > 0:
                    databox_info = "[nodule %s]" % slab.name

            message = "%s[codebox: %s][slab: %s]: %s" % (prefix, slab.Codebox.Name, slabInfo, info)

            s.failure_report.append(message)

    duration = datetime.now() - start_time

    if len(s.failure_report) > 0:
        s.t.Fail()
        print("\n".join(s.failure_report))
    outcome = "SUCCESS"
    if len(s.failure_report) > 0:
        outcome = "FAILURE"
    print(
        ("Ran {} slabs in {}\n"
        "{} -- {} Passed | {} Failed | {} Aborted | {} Raised").format(
            s.slab_count, duration,
            outcome, s.slab_passed, s.slab_failed, s.slab_aborted, s.slab_panicked,
        )
    )
