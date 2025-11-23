local myObj = {
  withArg(myarg):: {
    a: myarg,
  },
  withoutArg():: {
    b: 5,
  },
  withDefaultArg(myarg={ default: 3 }):: {
    c: myarg,
  },
};

{
  x: myObj,
}
