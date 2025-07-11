local myFunc(arg) = {
  assert std.isString(arg),
  x: arg,
};

{
  x: myFunc(1),
}
