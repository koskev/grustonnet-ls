local myVar = 5;
{
  x: if myVar == 5 then {} else {},
  [if myVar == 5 then 'z']: 5,
}
