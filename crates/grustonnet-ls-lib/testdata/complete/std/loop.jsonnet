local myObj = {
  key: 5,
};

local combined = myObj + { [x]: 1 for x in ['a'] };

{
  x: combined,
}
