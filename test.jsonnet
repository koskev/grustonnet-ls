local a = { orig: 1 };

local res = std.get(a, 'x', { backup: 3 });

{
  x: a,
  y: self.x,
  z: res.backup,
}
