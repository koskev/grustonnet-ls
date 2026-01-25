local myFunc(acc, arg) = {
  inner: acc.inner + arg,
};

local myFuncR(arg, acc) = {
  inner: acc.inner + arg,
};
local myArr = [1, 2, 3];

{
  l:: std.foldl(myFunc, myArr, { inner: 0 }),
  r:: std.foldr(myFuncR, myArr, { inner: 0 }),


  x: self.l,

}
