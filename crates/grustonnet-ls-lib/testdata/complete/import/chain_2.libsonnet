local next3 = import 'chain_3.libsonnet';
{
  two: (import 'chain_3.libsonnet'),
  two_local: next3,
}
