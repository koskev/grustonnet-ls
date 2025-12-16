{
  new():: {
    local outer = self,
    local _ = outer,

    withArg(arg):: self {
      key: arg,
    },

    withoutArg():: self {
      noArg: 1,
    },

    withInner():: {
      local inner = self,
      local _ = inner,
      innerVal: 0,

      withInnerFunc():: inner {
        innerVal: 5,
      },

      endInner():: outer {
        innerVal: inner.innerVal,
      },
    },


  },
  x: self.new(),

}
