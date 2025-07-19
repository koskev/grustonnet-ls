local myimport = import 'coolLib.libsonnet';

function(
  argone=(import 'coolLib.libsonnet'),
  argtwo={ argkey: 2 },
  argthree=myimport,
  argfour=myimport.libobject,
) {
  x: argone,
  y: argtwo,
  z: argthree,
  a: argfour,
}
