local myFunc(arg) = {
  key: arg,
};

local myObj = {
  objKey: 2,
};

{
  x: myFunc(
    myObj.objKey
  ),
}
