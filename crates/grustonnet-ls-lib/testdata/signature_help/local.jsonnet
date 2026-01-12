local myFunc(arg1, arg2) = arg1 + arg2;
local myFunc2(arg1, arg2) = arg1 + arg2;
{
  simple: myFunc(1, 2),
  nested: myFunc(myFunc2(1, 2), 3),
}
