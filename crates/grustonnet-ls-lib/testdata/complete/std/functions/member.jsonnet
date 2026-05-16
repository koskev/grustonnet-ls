local myArr = ['hello', 'world'];
local myObjTrue = if std.member(myArr, 'hello') then { trueKey: 1 } else { falseKey: 1 };
local myObjFalse = if std.member(myArr, 'foo') then { trueKey: 1 } else { falseKey: 1 };
{
  x: myObjTrue,
  y: myObjFalse,
}
