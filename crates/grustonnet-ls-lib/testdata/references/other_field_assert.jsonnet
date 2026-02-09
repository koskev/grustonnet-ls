local myVar = { a: 5 };
{
  local outerSelf = self,
  x: myVar,
  myKey::
    assert true;
    outerSelf.x,
}
