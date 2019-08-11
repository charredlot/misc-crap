from functools import singledispatch

import json


@singledispatch
def to_json(val):
    return json.dumps(val)
