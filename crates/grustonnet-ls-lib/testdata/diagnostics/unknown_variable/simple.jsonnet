local outerLocal = 5;
{
  local myLocal = 5,
  a: outerLocal,
  b: myLocal,
  c: invalid_var,
}
