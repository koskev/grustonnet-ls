local jlib = import 'lib/mylib.libsonnet';
local lib = import 'mylib.libsonnet';
{
  x: lib.localKey,
  y: jlib.libKey,

}
