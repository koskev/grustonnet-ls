local lib = import './stdlib.libsonnet';

local myVar = {
  x: 5,
};
[
  lib.myFunc(
    myVar.x
  ),
]
