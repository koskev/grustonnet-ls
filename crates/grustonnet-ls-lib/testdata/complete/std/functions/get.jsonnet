local myVar = {
  key: {
    inner: 1,
  },
  hidden:: {
    inner_hidden: 3,
  },
};
local fromLocal = std.get(myVar, 'key', default={ default: 5 }, inc_hidden=true);

{
  withVal: std.get(myVar, 'key', { default: 5 }),
  withDefault: std.get(myVar, 'nothere', { default: 5 }),
  withHidden: std.get(myVar, 'hidden', { default: 5 }, inc_hidden=true),
  withoutHidden: std.get(myVar, 'hidden', { default: 5 }, false),

  x:: fromLocal,
}
