local myArr = [[{ keyOne: 1 }], [{ keyTwo: 2 }]];

{
  flattened:: std.flattenArrays(myArr),

  x: self.flattened,
}
