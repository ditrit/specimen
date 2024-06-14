import yaml

import python.syaml as syaml
import python.focustree as focustree


def read_flag(node: yaml.Node):
    syaml.assert_is_mapping(node)
    flag_node = syaml.map_try_get_value(node, "flag")
    flag = focustree.Flag.NONE

    if flag_node is None:
        return flag

    name = ""
    both = False
    sentence = syaml.get_string(flag_node).split()
    for word in sentence:
        if word == "FOCUS":
            if flag == focustree.Flag.SKIP:
                both = True
            flag = focustree.Flag.FOCUS
            name = word
        elif word == "PENDING":
            if flag == focustree.Flag.FOCUS:
                both = True
            flag = focustree.Flag.SKIP
            name = word
        elif word.isupper():
            print(
                f'Warning: Unrecognised all uppercase flag "{word}". It has been ignored.'
            )
    if both:
        print(
            f"Warning: both FOCUS and PENDING have been found among the flags of a node. {name} has been kept."
        )

    return flag
