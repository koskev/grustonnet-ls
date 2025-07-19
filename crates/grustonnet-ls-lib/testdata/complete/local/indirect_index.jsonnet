local myObj = {
  first: {
    second: {
      third: 3,
    },
  },
};

local one = myObj.first;
local two = one.second;
local three = two.third;


{
  x: three,
}
