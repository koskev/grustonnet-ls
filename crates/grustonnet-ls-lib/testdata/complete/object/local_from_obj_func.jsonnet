local myLocal = {
  localKey: 1,
};

{
  normalKey: myLocal,
  funcKey():: myLocal,
  funcKey2(myArg):: myArg,
}
