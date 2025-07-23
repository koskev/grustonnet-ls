local myFunc = (import 'func.libsonnet')() + { key2: 2 };
{
  x: myFunc,
}
