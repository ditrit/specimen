import yaml


def assert_is_mapping(node: yaml.Node):
    if not is_mapping(node):
        raise ValueError("expected a mapping node")


def is_sequence(node: yaml.Node) -> bool:
    return node.tag == "tag:yaml.org,2002:seq"


def is_mapping(node: yaml.Node) -> bool:
    return node.tag == "tag:yaml.org,2002:map"


def is_string(node: yaml.Node) -> bool:
    return node.tag == "tag:yaml.org,2002:str"
