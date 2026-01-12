{
  myFunc(arg1, arg2):: arg1 + arg2,
  myFunc2(arg1, arg2):: arg1 + arg2,
  simple: self.myFunc(3, 4),
  nested: self.myFunc(self.myFunc2(1, 2), 3),
}
