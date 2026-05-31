local myObjOne = { one: 1 };
local myObjTwo = { two: 1 };
{
  x: (myObjOne + myObjTwo),
  y: self.x,
}
