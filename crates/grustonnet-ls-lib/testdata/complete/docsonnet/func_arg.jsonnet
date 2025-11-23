local __ = import 'docsonnet.libsonnet';
{
  funcs: {
    '#myFunc': __.fn(
      |||
        My Function
      |||,
      [
        __.arg('arg', __.T.any, help='Help Text'),
      ]
    ),
    myFunc(arg):: arg,
  },

  x: self.funcs.myFunc(1),
}
