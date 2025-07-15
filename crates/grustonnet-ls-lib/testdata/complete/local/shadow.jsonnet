local myVar = { one: 1 };

local middle = myVar;

local myVar = {
  two: middle,
};


{
  x: myVar,
}
