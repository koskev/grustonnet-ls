local myFunc(acc, arg) = {
  inner: acc.inner + arg,
};

local myFuncR(arg, acc) = {
  inner: acc.inner + arg,
};
local myArr = [1, 2, 3];

local concatObject(objs) = std.foldl(function(a, b) a + b, objs, {});
local retFunc(arg) = arg;

local inner = 'for checking diag crash with unused linter';

{
  l:: std.foldl(myFunc, myArr, { inner: 0 }),
  r:: std.foldr(myFuncR, myArr, { inner: 0 }),
  c:: concatObject([{ inner: 0 }]),
  u:: inner,


  x: self.l,
  y: retFunc(concatObject([{ inner: 0 }])).inner,

}
