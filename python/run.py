import enum
from datetime import datetime
import sys
import traceback
from typing import Callable, Dict, Any
import unittest

import python.focustree as focustree
from .file import File
from .nodule import Nodule


class FailStatus(enum.Enum):
    PRISTINE = 0
    FAILED = 1
    ABORTED = 2
    RAISED = 3


class SpecimenContext:
    def __init__(self, test_case: unittest.TestCase):
        self.test_case = test_case
        self.tile_count = 0
        self.tile_passed = 0
        self.tile_failed = 0
        self.tile_aborted = 0
        self.tile_raised = 0
        self.failure_report = []

        self.status = FailStatus.PRISTINE
        self.fail_info = ""

    def fail(self, info: str):
        self.status = FailStatus.FAILED
        self.fail_info = info

    def abort(self, info: str):
        self.status = FailStatus.ABORTED
        self.fail_info = info
        raise Exception()

    def expect_equal(self, value, wanted, context: str = ""):
        if value != wanted:
            if len(context) > 0:
                context = "(" + context + "): "
            self.fail(context + str(value))


def run(*data_file_list):
    return ioless_run(*data_file_list, stdout=sys.stdout)

def ioless_run(*data_file_list, stdout):
    class TestClass(unittest.TestCase):
        def run_function(self, testcase_function: type):
            flat_run(self, testcase_function, data_file_list, stdout)

    test_instance = TestClass()

    return test_instance.run_function


def flat_run(
    t: unittest.TestCase,
    test_function: Callable[[Dict[str, str]], None],
    data_file_list: list[File],
    stdout: Any,
):
    s = SpecimenContext(t)

    tree = []
    for file in data_file_list:
        try:
            tree.append(Nodule.parse_file(file))
        except Exception as e:
            print(f"{file.path}: {e}")

    valid_tree = []
    for nodule in tree:
        data_map = dict(filepath=[nodule.file_path])
        try:
            nodule.populate(data_map)
        except Exception as e:
            print("Exception", e)
        else:
            valid_tree.append(nodule)

    nodule_tree = Nodule(
        node=None,
        flag=focustree.Flag.NONE,
        be_leaf=False,
        file_path="",
        data_matrix=dict(),
        children=valid_tree,
    )
    flag_stat = focustree.FlagStat(0, 0)
    selected_leaves = focustree.extract_selected_leaves(nodule_tree, flag_stat, stdout)
    start_time = datetime.now()

    # Run all the selected slab
    for slab in selected_leaves:
        # Pass the tile to the testbox
        # - Manage the context (s, test start and test end)
        # - Catch any exception that might occure during the testbox call
        slab_location = slab.get_location()

        for index, tile in enumerate(slab):
            # Tile start
            s.status = FailStatus.PRISTINE
            s.fail_info = ""

            # Tile run
            try:
                test_function(s, **tile)
            except Exception as e:
                if s.status == FailStatus.ABORTED:
                    pass
                elif isinstance(e, AssertionError):
                    s.status = FailStatus.FAILED
                    s.fail_info = str(e)
                else:
                    report = "".join(map(str, traceback.format_exception(e)))

                    s.status = FailStatus.RAISED
                    s.fail_info = report

            # Tile end
            s.tile_count += 1
            attribute_name = {
                FailStatus.PRISTINE: "tile_passed",
                FailStatus.FAILED: "tile_failed",
                FailStatus.ABORTED: "tile_aborted",
                FailStatus.RAISED: "tile_raised",
            }[s.status]
            setattr(s, attribute_name, getattr(s, attribute_name) + 1)

            # summarize the failures
            if s.status != FailStatus.PRISTINE:
                word = {
                    FailStatus.FAILED: "FAIL",
                    FailStatus.ABORTED: "ABORT",
                    FailStatus.RAISED: "RAISE",
                }[s.status]

                message = "%s[%s][%d]: %s" % (word, slab_location, index, s.fail_info)

                s.failure_report.append(message)

    duration = datetime.now() - start_time

    if flag_stat.focus_count > 0 or flag_stat.skip_count > 0:
        message_list = []
        if flag_stat.focus_count > 0:
            message_list.append(f"{flag_stat.focus_count} focused node(s)")
        if flag_stat.skip_count > 0:
            message_list.append(f"{flag_stat.skip_count} pending node(s)")
        print(f"Encountered {' and '.join(message_list)}", file=stdout)

    outcome = "SUCCESS"
    if len(s.failure_report) > 0:
        print("\n".join(s.failure_report), file=stdout)
        outcome = "FAILURE"
    print(
        (
            "Ran {} tiles in {:.3g}ms\n"
            "{} -- {} Passed | {} Failed | {} Aborted | {} Raised"
        ).format(
            s.tile_count,
            duration.total_seconds() * 1000,
            outcome,
            s.tile_passed,
            s.tile_failed,
            s.tile_aborted,
            s.tile_raised,
        ),
        file=stdout,
    )
