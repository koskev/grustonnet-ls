local __ = import "../../complete/docsonnet/docsonnet.libsonnet";
{
  other: 9,
  '#myTest':: __.val(
    |||
      Description
    |||,
    __.T.any,
    self.other,
    ),
    myTest: 5,

}
