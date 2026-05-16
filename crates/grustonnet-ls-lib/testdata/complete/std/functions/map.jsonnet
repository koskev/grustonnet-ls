local myArr = [{ foo: 'foo' }, { bar: 'bar' }];

local myObj = std.map(function(elem) { key: elem }, myArr);
local myObjFlat = std.foldl(function(arg, acc) arg + acc, myObj, {});
{
  x: myObj[0].key,
  f: myObjFlat,
}
