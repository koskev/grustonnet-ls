local imported = import 'imported.libsonnet';

{
  x: imported.selfval.y,
}
