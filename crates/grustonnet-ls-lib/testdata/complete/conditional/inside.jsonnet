local myObj = {
  keyTrue: 1,
  keyFalse: 2,
};
{
  x: if
    myObj == 4
  then
    myObj.keyTrue
  else
    myObj.keyFalse,
}
