local myObj = {
  outer: {
    inner: 5,
  },
};

{
  x: myObj.outer.inner,
}
