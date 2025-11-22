local __ = import 'docsonnet.libsonnet';
{
  funcs: {
    '#myFunc': __.fn(
      |||
        My Function
      |||,
      [

      ]
    ),
    myFunc():: '42',
  },

  x: self.funcs.myFunc(),
}
