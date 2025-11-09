local myLib = import 'mylib.libsonnet';

{
  a: 5,
  x: myLib.dollarKey.objKey,
}
