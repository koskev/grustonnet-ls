{
  local selfvar = self,
  outer: 1,
  nested: {
    nestedkey: 2,
    x: selfvar.outer,
  },
}
