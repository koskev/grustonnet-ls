local myVar = [{ one: 1, two: 2 }, { second_object: 1 }];
{
  x: std.map(
    function(x)
      x,
    [{ one: 1, two: 2 }, { second_object: 1 }]
  ),
  y: std.map(
    function(y)
      y,
    myVar
  ),
}
