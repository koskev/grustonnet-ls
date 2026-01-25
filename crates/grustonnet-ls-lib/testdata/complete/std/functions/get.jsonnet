local myVar = {
  key: {
    inner: 1,
  },
  hidden:: {
    inner_hidden: 3,
  },
};
local myFunc(arg) = {
  key: {
    inner: arg,
  },
};
local fromLocal = std.get(myVar, 'key', default={ default: 5 }, inc_hidden=true);

{
  withVal: std.get(myVar, 'key', { default: 5 }),
  withDefault: std.get(myVar, 'nothere', { default: 5 }),
  withHidden: std.get(myVar, 'hidden', { default: 5 }, inc_hidden=true),
  withoutHidden: std.get(myVar, 'hidden', { default: 5 }, false),

  withFunction: std.get(myFunc(1), 'key'),

  x:: fromLocal,
}
