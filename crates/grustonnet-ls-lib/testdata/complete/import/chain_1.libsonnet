local next2 = import 'chain_2.libsonnet';
{
  one: (import 'chain_2.libsonnet'),
  one_local: next2,
}
