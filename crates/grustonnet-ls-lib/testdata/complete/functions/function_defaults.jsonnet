local myFunc(argone=1, argtwo={ argkey: 2 }) = {
  x: argone,
  y: argtwo,
};

{
  z: myFunc(1, 2),
}
