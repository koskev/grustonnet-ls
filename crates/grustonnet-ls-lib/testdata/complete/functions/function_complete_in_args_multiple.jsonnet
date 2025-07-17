local myFunc(arg1, arg2, arg3) = {
  key1: arg1,
  key2: arg2,
  key3: arg3,
};

local myObj1 = {
  objKey1: 1,
};

local myObj2 = {
  objKey2: 2,
};

local myObj3 = {
  objKey3: 3,
};

{
  x: myFunc(
    arg1=myObj1.objKey1,
    arg2=myObj2.objKey2,
    arg3=myObj3.objKey3,
  ),
  y: myFunc(
    myObj1.objKey1, // 1
    myObj2.objKey2, // 2
    myObj3.objKey3, // 3
  ),
}
