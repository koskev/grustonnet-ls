{
  new():: {

    withArg(arg):: self {
      key: arg,
    },

    withoutArg():: self {
      noArg: 1,
    },


  },
  x: self.new(),

}
